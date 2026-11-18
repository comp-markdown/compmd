// `compmd-pack dev` (docs/writing-a-pack.md, decision 3): the test page, served from memory, never written into the
// pack. It renders the pack's `demo.comp.md` with the pack as a folder pack, in the strict sandbox, as a host renders
// it, in host mode (each use in a strict frame of its own, which takes the height of what it draws), with the
// document's diagnostics under it and a light and dark switch; saving a source builds its module again.

import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { compmd } from "@compmd/vite";
import { build, createServer, type Plugin, type ViteDevServer } from "vite";
import { buildModules, moduleConfig } from "./build.ts";
import type { Pack } from "./pack.ts";

/** The document the test page renders, in the pack's folder. */
export const DEMO = "demo.comp.md";

const PAGE = "/@compmd-pack/page.ts";
const RESOLVED_PAGE = "\0compmd-pack-page";

/** The test page's HTML. */
export function pageHtml(title: string): string {
  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${title.replace(/[<&]/g, (c) => (c === "<" ? "&lt;" : "&amp;"))}</title>
    <style>
      :root { color-scheme: light dark; }
      body { font: 16px/1.5 system-ui, sans-serif; margin: 2rem auto; max-width: 48rem; padding: 0 1rem; background: Canvas; color: CanvasText; }
      header { display: flex; justify-content: flex-end; gap: 0.5rem; align-items: center; }
      #diagnostics { color: #d1242f; }
    </style>
  </head>
  <body>
    <header><label for="scheme">Colour scheme</label> <select id="scheme"><option value="light dark">The reader's</option><option value="light">Light</option><option value="dark">Dark</option></select></header>
    <div id="page"></div>
    <ul id="diagnostics"></ul>
    <script type="module" src="${PAGE}"></script>
  </body>
</html>
`;
}

/** The test page's script: the document processed with the pack as a host gives it, and rendered in the sandbox. */
export const PAGE_SCRIPT = `import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "/${DEMO}?raw";

const page = document.getElementById("page");
const list = document.getElementById("diagnostics");
const scheme = document.getElementById("scheme");
// A component that fails again when it draws again (a new scheme, say) reports the same failure: it shows once.
const shown = new Set();
const show = (line) => {
  if (shown.has(line)) return;
  shown.add(line);
  list.append(Object.assign(document.createElement("li"), { textContent: line }));
};

await init();
const opened = new OpenDocument({ ...request, path: "${DEMO}", files: { "${DEMO}": text }, now: new Date().toISOString() });
// Host mode: each use in a strict frame of its own, sized to what it draws, as an app shows a document.
const view = createSandbox({ ...sandbox, page: "host" }).open(page, { components: opened.components });
view.on("error", ({ component, code, reason, message }) => show(\`\${code ?? reason}: \${component}: \${message}\`));
view.render(opened.expanded);
for (const { code, message } of opened.diagnostics) show(\`\${code}: \${message}\`);
scheme.addEventListener("change", () => {
  document.documentElement.style.colorScheme = scheme.value;
  view.scheme();
});
`;

/** The configuration the test page's `compmd()` gets: the pack as a folder pack, and what the pack's own file adds. */
export async function pageConfig(pack: Pack): Promise<Record<string, unknown>> {
  const file = join(pack.dir, "compmd.config.json");
  const own = existsSync(file) ? (JSON.parse(await readFile(file, "utf8")) as { packs?: Record<string, unknown> }) : {};
  return { ...own, packs: { "./": {}, ...own.packs } };
}

/**
 * The test page: its HTML at `/`, and its script, whose imports of `@compmd/processor` and `@compmd/sandbox` resolve
 * from this package, which depends on them, rather than from the pack, which doesn't.
 */
function testPage(pack: Pack): Plugin {
  const here = fileURLToPath(import.meta.url);
  return {
    name: "compmd-pack:page",
    resolveId(id, importer) {
      if (id === PAGE) return RESOLVED_PAGE;
      if (importer === RESOLVED_PAGE && id.startsWith("@compmd/")) return this.resolve(id, here, { skipSelf: true });
      return undefined;
    },
    load(id) {
      return id === RESOLVED_PAGE ? PAGE_SCRIPT : undefined;
    },
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const path = new URL(request.url ?? "/", "http://localhost").pathname;
        if (path !== "/" && path !== "/index.html") {
          next();
          return;
        }
        void server.transformIndexHtml(path, pageHtml(pack.manifest.name)).then((html) => {
          response.setHeader("Content-Type", "text/html");
          response.end(html);
        }, next);
      });
    },
  };
}

/** Builds a module again when its source changes; the sandbox's plugin then loads the page again. */
function rebuild(pack: Pack): Plugin {
  const sources = join(pack.dir, "src");
  return {
    name: "compmd-pack:rebuild",
    configureServer(server) {
      server.watcher.add(sources);
      server.watcher.on("change", (file) => {
        if (!file.startsWith(sources + sep) || /\.test\.[jt]sx?$/.test(file)) return;
        for (const module of pack.modules) void build(moduleConfig(pack, module));
      });
    },
  };
}

/**
 * Builds `pack`'s modules, then serves its test page on `port` (Vite's default without one).
 *
 * @throws when `demo.comp.md` isn't in the pack's folder.
 * @example
 * const server = await servePack(await readPack("."), { port: 4200 });
 * server.printUrls();
 */
export async function servePack(pack: Pack, options: { port?: number } = {}): Promise<ViteDevServer> {
  if (!existsSync(join(pack.dir, DEMO))) throw new Error(`${DEMO} isn't in ${pack.dir}: the test page renders it, a document that imports the pack and uses it`);
  await buildModules(pack);
  const server = await createServer({
    configFile: false,
    root: pack.dir,
    logLevel: "info",
    appType: "custom",
    server: options.port === undefined ? {} : { port: options.port, strictPort: true },
    plugins: [compmd({ config: await pageConfig(pack) }), testPage(pack), rebuild(pack)],
  });
  await server.listen();
  return server;
}
