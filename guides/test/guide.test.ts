// The guides' chapters against what they claim, book by book: each block of code marked `snippet=<file>` is that
// example file's text, each example file is shown, each CompMD document in a chapter processes without diagnostics
// (with the packs it imports from npm), the contents lists every chapter at its number, and the examples build and run
// in Chrome and WebKit as the chapters say.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { cp, mkdir, rm } from "node:fs/promises";
import { createServer as createHttpServer, type Server } from "node:http";
import { createRequire } from "node:module";
import { join } from "node:path";
import { execPath } from "node:process";
import { gunzipSync } from "node:zlib";
import { initSync, OpenDocument, process, type PackRelease } from "@compmd/processor";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, preview, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "@compmd/vite";

const GUIDES = join(import.meta.dirname, "..");
const BOOKS = ["authors", "developers"];

/** The fenced blocks of `markdown`: their info string and their text. */
function blocksOf(markdown: string): { info: string; text: string }[] {
  const blocks: { info: string; text: string }[] = [];
  const lines = markdown.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const opener = /^(`{3,})(.*)$/.exec(lines[i] ?? "");
    if (!opener) continue;
    const [, fence = "", info = ""] = opener;
    const end = lines.findIndex((line, j) => j > i && line.startsWith(fence) && line.trim() === fence);
    blocks.push({ info: info.trim(), text: lines.slice(i + 1, end).join("\n") });
    i = end;
  }
  return blocks;
}

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

/** The text files of an example's folder, by their path in it. */
function filesOf(folder: string): Record<string, string> {
  return Object.fromEntries(
    readdirSync(folder, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => {
        const path = join(entry.parentPath, entry.name);
        return [path.slice(folder.length + 1).split("\\").join("/"), readFileSync(path, "utf8")];
      }),
  );
}

/** The integrity npm publishes for the Mermaid pack's versions the chapters pin. */
const PINNED: Record<string, string> = { "0.1.1": "sha512-EwevqkkUdxOsFLHV85W74wEL+LoyZhaZkmnj8bJax3zLG48uuiGrezFTixCdlS6VQC/GHVU8Ra0vVXjeFftc7Q==" };

/**
 * The packs a chapter's documents import from npm, as a host that fetches them gives them: the Mermaid pack, from this
 * repository's `packs/mermaid/`, its built module given by its place, with the integrity npm publishes for its version.
 */
function registryPacks(): Record<string, PackRelease[]> {
  const folder = join(GUIDES, "../packs/mermaid");
  const manifest = readFileSync(join(folder, "package.json"), "utf8");
  const { name, version } = JSON.parse(manifest) as { name: string; version: string };
  const files = { "package.json": manifest, "index.comp.md": readFileSync(join(folder, "index.comp.md"), "utf8"), "dist/mermaid.js": "" };
  return { [name]: [{ version, integrity: PINNED[version] ?? "sha512-from-the-repository", files }] };
}

describe.each(BOOKS)("the %s' guide", (book) => {
  const folder = join(GUIDES, book);
  const examples = join(folder, "examples");
  /** Whether a block shows a file of an example that is a pack: a folder with a `package.json`. */
  const inPack = (info: string) => {
    const file = /\bsnippet=(\S+)/.exec(info)?.[1];
    return file !== undefined && existsSync(join(examples, file.split("/")[0] ?? "", "package.json"));
  };
  const chapters = readdirSync(folder).filter((name) => /^\d\d-.+\.md$/.test(name)).sort();

  it("numbers each chapter as its contents does", () => {
    const contents = readFileSync(join(folder, "README.md"), "utf8");
    const numbered = new Map([...contents.matchAll(/^(\d+)\. \[[^\]]+\]\((\d\d-[^)]+\.md)\)/gm)].map(([, number, file]) => [file, Number(number)]));
    for (const chapter of chapters) expect(Number(chapter.slice(0, 2)), chapter).toBe(numbered.get(chapter));
  });

  for (const chapter of chapters) {
    const markdown = readFileSync(join(folder, chapter), "utf8");

    it(`${chapter}: its code is its examples' files`, () => {
      for (const { info, text } of blocksOf(markdown)) {
        const file = /\bsnippet=(\S+)/.exec(info)?.[1];
        if (file === undefined) continue;
        expect(text, file).toBe(readFileSync(join(examples, file), "utf8").replace(/\n$/, ""));
      }
    });

    // A pack's own document imports its built module, so it is checked where the pack is built (below). An example's
    // document has the other files of its folder, which it reads; any other block is a document on its own.
    it(`${chapter}: its CompMD documents process without diagnostics`, () => {
      for (const { info, text } of blocksOf(markdown).filter(({ info }) => info.split(" ")[0] === "md" && !inPack(info))) {
        const file = /\bsnippet=(\S+)/.exec(info)?.[1];
        const [name = "", ...rest] = file?.split("/") ?? [];
        const path = rest.length > 0 ? rest.join("/") : "doc.comp.md";
        const files = file === undefined ? { [path]: text } : { ...filesOf(join(examples, name)), [path]: text };
        const { diagnostics } = process({ path, files, packs: registryPacks(), now: "2026-10-31T09:00:00Z" });
        expect(diagnostics.map(({ code, message }) => `${code} ${message}`), `${chapter}: ${info}`).toEqual([]);
      }
    });
  }

  it("shows every file of its examples", () => {
    const shown = new Set(chapters.flatMap((chapter) => blocksOf(readFileSync(join(folder, chapter), "utf8")).map(({ info }) => /\bsnippet=(\S+)/.exec(info)?.[1])));
    const files = existsSync(examples)
      ? readdirSync(examples, { recursive: true, withFileTypes: true })
          .filter((entry) => entry.isFile())
          .map((entry) => join(entry.parentPath, entry.name).slice(examples.length + 1).split("\\").join("/"))
      : [];
    for (const file of files) expect(shown.has(file), file).toBe(true);
  });

  it("lists each chapter in its contents, which links only to chapters that are there", () => {
    const contents = readFileSync(join(folder, "README.md"), "utf8");
    const linked = [...contents.matchAll(/\]\((\d\d-[^)]+\.md)\)/g)].map((match) => match[1]);
    expect(linked).toEqual(chapters);
    for (const file of linked) expect(existsSync(join(folder, file ?? ""))).toBe(true);
  });
});

it("lists every book on the shelf", () => {
  const shelf = readFileSync(join(GUIDES, "README.md"), "utf8");
  for (const book of BOOKS) expect(shelf).toContain(`](${book}/README.md)`);
});

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe("the developers' quick start", () => {
  const root = join(GUIDES, "developers/examples/quick-start");
  const out = join(import.meta.dirname, ".out/quick-start");
  const port = 4201;
  let server: PreviewServer;

  beforeAll(async () => {
    // A cache of its own: the plugin empties the frames' folder in it, which test files building at once would share.
    const config = { root, configFile: false as const, logLevel: "warn" as const, cacheDir: join(out, ".vite"), plugins: [compmd()], build: { outDir: join(out, "site"), emptyOutDir: true, target: "es2024" } };
    await build(config);
    server = await preview({ ...config, preview: { port, strictPort: true } });
  });

  afterAll(async () => {
    await server.close();
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("renders the document, and follows its input as the chapter says", async () => {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`http://localhost:${String(port)}/`);
      const frame = page.frameLocator("iframe");
      await expect.poll(() => frame.locator("body").innerText(), { timeout: 20_000 }).toContain("1 of 4 done");
      expect(await frame.locator("li").allInnerTexts()).toEqual(["Write the outline", "Draft the first part", "Ask for a review", "Publish"]);
      await frame.locator("input[type=number]").fill("3");
      await expect.poll(() => frame.locator("body").innerText()).toContain("3 of 4 done");
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});

describe("the developers' pack", () => {
  const example = join(GUIDES, "developers/examples/progress-pack");
  const copy = join(import.meta.dirname, ".out/progress-pack");
  const cli = join(GUIDES, "node_modules/@compmd/pack/dist/cli.js");

  // A copy, so what the build writes stays out of the examples; it finds @compmd/pack and its tools in the guides'.
  beforeAll(async () => {
    await rm(copy, { recursive: true, force: true });
    await cp(example, copy, { recursive: true });
  });

  it("builds and passes the checks before publishing, as the chapter says", () => {
    execFileSync(execPath, [cli, "build"], { cwd: copy, stdio: "pipe" });
    expect(readdirSync(join(copy, "dist"))).toContain("progress.js");
  });

  it("passes its component's tests", () => {
    execFileSync(execPath, [cli, "test"], { cwd: copy, stdio: "pipe" });
  });

  it("is a pack whose document processes without diagnostics, as a host reads it", () => {
    const file = (path: string) => readFileSync(join(copy, path), "utf8");
    const pack = { version: "0.1.0", integrity: "sha512-from-the-build", files: { "package.json": file("package.json"), "index.comp.md": file("index.comp.md"), "dist/progress.js": file("dist/progress.js") } };
    const document = "---\nmd:\n  import:\n    - from: \"@you/progress@0\"\n      names: [progress]\n---\n\n:: progress[3 of 4 done] {value=3 max=4}\n";
    const opened = new OpenDocument({ path: "doc.comp.md", files: { "doc.comp.md": document }, packs: { "@you/progress": [pack] }, now: "2026-10-31T09:00:00Z" });
    expect(opened.diagnostics.map(({ code, message }) => `${code} ${message}`)).toEqual([]);
    expect(opened.components.map(({ name, module }) => [name, module?.path])).toEqual([["progress", "dist/progress.js"]]);
    opened.free();
  });
});

/** The sum of the sizes of the files in a gzipped tar: the unpacked size npm's metadata states. */
function unpackedSize(gzipped: Buffer): number {
  const tar = gunzipSync(gzipped);
  let total = 0;
  for (let at = 0; at + 512 <= tar.length; ) {
    if (tar[at] === 0) break;
    const size = parseInt(tar.subarray(at + 124, at + 136).toString("ascii").replace(/\0.*$/, "").trim() || "0", 8);
    if (tar[at + 156] === 0x30 || tar[at + 156] === 0) total += size;
    at += 512 + Math.ceil(size / 512) * 512;
  }
  return total;
}

describe("the developers' packs from npm", () => {
  const root = join(GUIDES, "developers/examples/packs-from-npm");
  const out = join(import.meta.dirname, ".out/packs-from-npm");
  const mermaid = join(GUIDES, "../packs/mermaid");
  const port = 4202;
  const registryPort = 4203;
  let server: PreviewServer;
  let registry: Server;
  const requests: string[] = [];

  // The Mermaid pack as npm serves it, from a local registry: its tarball as `pnpm pack` writes it, with its metadata.
  // The example is built with its own configuration, but for its registry, which is the local one.
  beforeAll(async () => {
    await rm(out, { recursive: true, force: true });
    await mkdir(join(out, "packages"), { recursive: true });
    execFileSync("pnpm", ["pack", "--pack-destination", join(out, "packages")], { cwd: mermaid, stdio: "pipe" });
    const tarball = readFileSync(join(out, "packages", readdirSync(join(out, "packages"))[0] ?? ""));
    const { name, version } = JSON.parse(readFileSync(join(mermaid, "package.json"), "utf8")) as { name: string; version: string };
    const dist = { tarball: `http://127.0.0.1:${String(registryPort)}/tarball.tgz`, integrity: `sha512-${createHash("sha512").update(tarball).digest("base64")}`, unpackedSize: unpackedSize(tarball) };
    registry = createHttpServer((request, response) => {
      const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
      requests.push(path);
      if (path === "/tarball.tgz") return response.writeHead(200, { "content-type": "application/octet-stream" }).end(tarball);
      if (path !== `/${name}`) return response.writeHead(404).end();
      response.writeHead(200, { "content-type": "application/vnd.npm.install-v1+json" }).end(JSON.stringify({ name, versions: { [version]: { version, dist } } }));
    });
    await new Promise<void>((resolve) => registry.listen(registryPort, "127.0.0.1", resolve));
    const written = JSON.parse(readFileSync(join(root, "compmd.config.json"), "utf8")) as Record<string, unknown>;
    const plugin = compmd({ config: { ...written, registries: { "*": `http://127.0.0.1:${String(registryPort)}/` } } });
    const config = { root, configFile: false as const, logLevel: "warn" as const, cacheDir: join(out, ".vite"), plugins: [plugin], build: { outDir: join(out, "site"), emptyOutDir: true, target: "es2024" } };
    await build(config);
    server = await preview({ ...config, preview: { port, strictPort: true } });
  });

  afterAll(async () => {
    await server.close();
    registry.closeAllConnections();
    await new Promise((resolve) => registry.close(resolve));
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("fetches the pack the document imports through the app's proxy, checks it, and draws its diagram", async () => {
      const page = await browser.newPage();
      const errors: string[] = [];
      const warnings: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      page.on("console", (message) => {
        if (message.type() === "warning") warnings.push(message.text());
      });
      await page.goto(`http://localhost:${String(port)}/`);
      const drawn = page.frameLocator("iframe").locator('[role="img"][aria-label="A change is written, reviewed, merged, then released"] svg');
      await expect.poll(() => drawn.count(), { timeout: 30_000 }).toBe(1);
      expect(await drawn.innerHTML()).toContain("Released");
      expect(requests).toContain("/tarball.tgz");
      expect(errors).toEqual([]);
      expect(warnings).toEqual([]);
      await page.close();
    });
  });
});
