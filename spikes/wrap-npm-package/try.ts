// CMD-8's spike (docs/wrapping-an-npm-package.md, *Trying it on four packages*): builds each pack here as
// `compmd-pack build` does, then opens its demo in Chrome and WebKit as `compmd-pack dev` serves it (host mode, each use
// in a strict frame of its own), and writes what was built, what drew, what the strict policy refused and what the page
// reported to `results.json`, with a screenshot of each page in `screenshots/`.
//
//   pnpm --dir spikes/wrap-npm-package try [<pack folder>...]
//   pnpm --dir spikes/wrap-npm-package try layerchart --serve     # its test page left running, for a person

import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { gzipSync } from "node:zlib";
import { readPack, type Pack } from "@compmd/pack";
import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { chromium, webkit, type BrowserType, type LaunchOptions, type Page } from "playwright";
import { build, createServer, type Plugin } from "vite";
// The pack tool's own build and test page, which its package doesn't export. The spike compiles a Svelte package's
// sources, which `compmd-pack build` doesn't yet (decision 10), so it can't call `servePack`, which builds first.
import { moduleConfig } from "../../npm-packages/pack/src/build.ts";
import { PAGE_SCRIPT, pageConfig, pageHtml } from "../../npm-packages/pack/src/dev.ts";

const HERE = import.meta.dirname;
const PACKS = ["react-qr-code", "layerchart", "model-viewer", "qrcode"];
const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];
/** How long a page has to draw every use of its demo. */
const DRAW_MS = 20_000;

interface Wrap {
  kind: "react" | "svelte" | "web-component" | "library";
}

interface Built {
  ok: boolean;
  error?: string;
  bytes?: number;
  gzipBytes?: number;
  /** What the module still imports: the runtimes the frames' import map gives. */
  imports?: string[];
}

/** A use's frame: its height in the page, and its drawing's size, in CSS pixels. */
interface Size {
  width: number;
  height: number;
}

interface UseFrame {
  frameHeight: number;
  /** The inside of the use's box, the component's `target`, which the use's common params size (spec §9.2). */
  box: Size | null;
  /** What is drawn: an SVG's `viewBox` as it is scaled into the element, else the element. */
  drawing: Size | null;
  /** How the drawing sits in its box (docs/writing-a-pack.md, decision 8). */
  fit: "fills both" | "fits width" | "fits height" | "inside" | "overflows" | "none";
  /** How far the frame's document goes past the frame: anything above 0 scrolls. */
  overflow: Size | null;
  /** The start of the frame's markup, when it didn't draw. */
  markup?: string;
}

interface Opened {
  uses: number;
  /** The uses whose drawing has a size, in a frame with a height. */
  drawn: number;
  frames: UseFrame[];
  refused: string[];
  errors: string[];
  diagnostics: string[];
}

const wrapOf = (dir: string): Wrap => JSON.parse(readFileSync(join(dir, "compmd.wrap.json"), "utf8")) as Wrap;

/** The pack's modules built as `compmd-pack build` builds them, a Svelte package's sources compiled first. */
async function buildPack(pack: Pack, wrap: Wrap): Promise<Built> {
  try {
    // A Svelte package's CSS goes into the module, which injects it (decision 10): a frame loads only the module.
    const plugins = wrap.kind === "svelte" ? [svelte({ configFile: false, emitCss: false, compilerOptions: { css: "injected" } })] : [];
    for (const module of pack.modules) await build({ ...moduleConfig(pack, module), plugins });
  } catch (error) {
    return { ok: false, error: (error instanceof Error ? error.message : String(error)).split("\n")[0] };
  }
  const out = join(pack.dir, ...(pack.modules[0]?.out ?? "").split("/"));
  const code = readFileSync(out);
  // Module syntax only: `import"x"` and `from"x"` after a statement's end, never the same words inside a string.
  const imports = [...new Set([...code.toString("utf8").matchAll(/(?:^|[;}])\s*import\s*(?:[^"';]*?from\s*)?"([^"./][^"]*)"/g)].map((match) => match[1] ?? ""))];
  return { ok: true, bytes: statSync(out).size, gzipBytes: gzipSync(code).length, imports };
}

/** The test page of `compmd-pack dev`, its script's `@compmd/` imports resolved from here. */
function testPage(pack: Pack): Plugin {
  const PAGE = "/@spike/page.ts";
  const RESOLVED = "\0spike-page";
  return {
    name: "spike:page",
    resolveId(id, importer) {
      if (id === PAGE) return RESOLVED;
      if (importer === RESOLVED && id.startsWith("@compmd/")) return this.resolve(id, join(HERE, "try.ts"), { skipSelf: true });
      return undefined;
    },
    load: (id) => (id === RESOLVED ? PAGE_SCRIPT : undefined),
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const path = new URL(request.url ?? "/", "http://localhost").pathname;
        if (path !== "/") {
          next();
          return;
        }
        void server.transformIndexHtml(path, pageHtml(pack.manifest.name).replace("/@compmd-pack/page.ts", PAGE)).then((html) => {
          response.setHeader("Content-Type", "text/html");
          response.end(html);
        }, next);
      });
    },
  };
}

/** In a use's frame: the inside of its box, and what is drawn in it. Runs in the page, so it is self-contained. */
function measureUse(): { box: Size; drawing: Size; overflow: Size } | null {
  const box = document.querySelector(".compmd-component");
  const element = box?.querySelector("svg, canvas");
  if (!box || !element) return null;
  // How far the frame's document goes past the frame, which a reader sees as scroll bars.
  const root = document.documentElement;
  const overflow = { width: root.scrollWidth - root.clientWidth, height: root.scrollHeight - root.clientHeight };
  const style = getComputedStyle(box);
  const inside = {
    width: box.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight),
    height: box.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom),
  };
  const shown = element.getBoundingClientRect();
  let drawing = { width: shown.width, height: shown.height };
  // An SVG scaled into its element (`meet`, the default) draws its `viewBox` only as large as both sides allow.
  if (element instanceof SVGSVGElement && element.viewBox.baseVal.width > 0 && element.getAttribute("preserveAspectRatio") !== "none") {
    const view = element.viewBox.baseVal;
    const scale = Math.min(shown.width / view.width, shown.height / view.height);
    drawing = { width: view.width * scale, height: view.height * scale };
  }
  const round = (size: Size) => ({ width: Math.round(size.width), height: Math.round(size.height) });
  return { box: round(inside), drawing: round(drawing), overflow };
}

/** How a drawing sits in its box, a pixel either way counting as equal. */
function fitOf(measured: { box: Size; drawing: Size; overflow: Size } | null): UseFrame["fit"] {
  if (!measured) return "none";
  const { box, drawing } = measured;
  if (drawing.width > box.width + 1 || drawing.height > box.height + 1) return "overflows";
  const width = Math.abs(drawing.width - box.width) <= 1;
  const height = Math.abs(drawing.height - box.height) <= 1;
  return width && height ? "fills both" : width ? "fits width" : height ? "fits height" : "inside";
}

/** The demo in `page`, once every use drew or `DRAW_MS` passed. */
async function openDemo(page: Page, url: string, uses: number): Promise<Opened> {
  const refused: string[] = [];
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    const text = message.text();
    if (/Content.Security.Policy|Refused to/i.test(text)) refused.push(text);
    // The test page has no icon, which Chrome asks for: not a component's.
    else if (message.type() === "error" && !message.location().url.endsWith("/favicon.ico")) errors.push(`${text} (${message.location().url})`);
  });
  await page.goto(url);
  const useFrames = async (): Promise<UseFrame[]> =>
    Promise.all(
      page.frames().slice(1).map(async (frame) => {
        const frameHeight = (await (await frame.frameElement()).boundingBox())?.height ?? 0;
        const measured = await frame.evaluate(measureUse).catch(() => null);
        return { frameHeight, box: measured?.box ?? null, drawing: measured?.drawing ?? null, fit: fitOf(measured), overflow: measured?.overflow ?? null };
      }),
    );
  const isDrawn = (use: UseFrame) => use.frameHeight > 0 && (use.drawing?.width ?? 0) > 0 && (use.drawing?.height ?? 0) > 0;
  const until = Date.now() + DRAW_MS;
  let frames = await useFrames();
  while (frames.filter(isDrawn).length < uses && Date.now() < until) {
    await page.waitForTimeout(250);
    frames = await useFrames();
  }
  const diagnostics = await page.locator("#diagnostics li").allInnerTexts();
  const useFrameList = page.frames().slice(1);
  for (const [index, use] of frames.entries()) {
    if (!isDrawn(use)) use.markup = (await useFrameList[index]?.locator("body").innerHTML().catch(() => ""))?.slice(0, 1500);
  }
  return { uses, drawn: frames.filter(isDrawn).length, frames, refused: [...new Set(refused)], errors: [...new Set(errors)], diagnostics };
}

const results: Record<string, { built: Built; opened: Record<string, Opened> }> = {};
/**
 * `--serve`: build the chosen packs as above and leave each test page running, opening no browser, for a person to
 * look at. `compmd-pack dev` can't yet, for a Svelte package (decision 10): it builds without Svelte's plugin.
 */
const SERVE = process.argv.includes("--serve");
const named = process.argv.slice(2).filter((argument) => argument !== "--serve");
const chosen = named.length > 0 ? named : PACKS;
mkdirSync(join(HERE, "screenshots"), { recursive: true });

for (const [index, name] of chosen.entries()) {
  const dir = join(HERE, name);
  const pack = await readPack(dir);
  const built = await buildPack(pack, wrapOf(dir));
  results[name] = { built, opened: {} };
  console.log(`${name}: ${built.ok ? `built, ${String(built.gzipBytes)} bytes gzipped, importing ${built.imports?.join(", ") || "nothing"}` : `not built: ${built.error ?? ""}`}`);
  if (!built.ok) continue;
  const port = 4230 + index;
  const server = await createServer({ configFile: false, root: dir, logLevel: "warn", appType: "custom", server: { port, strictPort: true }, plugins: [compmd({ config: await pageConfig(pack) }), testPage(pack)] });
  await server.listen();
  if (SERVE) {
    console.log(`  its test page: http://localhost:${String(port)}/ (Ctrl-C stops it; a change to its sources needs a new run)`);
    continue;
  }
  const uses = readFileSync(join(dir, "demo.comp.md"), "utf8").split("\n").filter((line) => line.startsWith(":: ")).length;
  for (const [browserName, type, options] of BROWSERS) {
    const browser = await type.launch({ headless: true, ...options });
    // Tall enough for every demo, so every frame is on screen and painted when the screenshot is taken.
    const page = await browser.newPage({ viewport: { width: 900, height: 2600 } });
    const opened = await openDemo(page, `http://localhost:${String(port)}/`, uses);
    await page.screenshot({ path: join(HERE, "screenshots", `${name}-${browserName.toLowerCase()}.png`), fullPage: true });
    results[name].opened[browserName] = opened;
    const size = (s: Size) => `${String(s.width)}×${String(s.height)}`;
    const sizes = opened.frames.map((use) => (use.drawing && use.box ? `\n    ${size(use.drawing)} in a ${size(use.box)} box: ${use.fit}${use.overflow && (use.overflow.width > 0 || use.overflow.height > 0) ? `; the frame scrolls by ${size(use.overflow)}` : ""}` : "\n    nothing drawn")).join("");
    console.log(`  ${browserName}: ${String(opened.drawn)} of ${String(opened.uses)} drawn; ${String(opened.refused.length)} refused; ${String(opened.errors.length)} errors; diagnostics: ${opened.diagnostics.join(" | ") || "none"}${sizes}`);
    await browser.close();
  }
  await server.close();
}

if (!SERVE) writeFileSync(join(HERE, "results.json"), `${JSON.stringify(results, null, 2)}\n`);
