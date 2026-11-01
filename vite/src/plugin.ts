// The Vite plugin (docs/vite.md): reads the configuration and the packs at build start, gives the app `virtual:compmd`,
// builds the sandbox's frame documents and libraries, serves them and the packs' built modules under `compmd/` in dev,
// and writes them into the build.

import { existsSync } from "node:fs";
import type { ServerResponse } from "node:http";
import { readdir, readFile, rm } from "node:fs/promises";
import { extname, join, relative, sep } from "node:path";
import { buildFrames, buildLibraries, libraryDefinition, readLibraries, RUNTIMES, type LibraryDefinition } from "@compmd/sandbox/build";
import type { Plugin, ResolvedConfig } from "vite";
import { loadApp, type App } from "./app.ts";

/** The plugin's options. */
export interface CompmdOptions {
  /** The folder of the app, under its base, where the sandbox and the packs' files are served; `compmd/` by default. */
  folder?: string;
}

const VIRTUAL = "virtual:compmd";
const RESOLVED = `\0${VIRTUAL}`;
const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".map": "application/json" };

/**
 * The app's request with the frames' runtimes and libraries in its policy (spec §7.6), so the processor reports a
 * runtime they don't have (W102) and a pack's `peerDependencies` they can't meet (W121).
 */
export function requestWith(app: App, definition: LibraryDefinition | undefined): App["request"] {
  if (!definition) return app.request;
  const named = ({ name, version }: { name: string; version: string }) => ({ name, version });
  return { ...app.request, policy: { ...app.request.policy, runtimes: definition.runtimes.map(named), libraries: definition.libraries.map(named) } };
}

/**
 * The code of `virtual:compmd`: the processor's request, and the sandbox's options with the trusted components the
 * app's own build compiles; with the frames' `definition`, the request's policy says what they provide.
 */
export function moduleCode(app: App, folder: string, definition?: LibraryDefinition): string {
  const bindings = new Set(app.trusted.map(({ binding }) => binding));
  const lines = [...bindings].filter((binding) => binding !== "plain").map((binding) => `import { ${binding} } from "@compmd/vite/${binding}";`);
  app.trusted.forEach(({ file }, i) => lines.push(`import c${String(i)} from ${JSON.stringify(file)};`));
  const trusted = app.trusted.map(({ name, binding }, i) => `${JSON.stringify(name)}: ${binding === "plain" ? `c${String(i)}` : `${binding}(c${String(i)})`}`);
  lines.push(
    `export const request = ${JSON.stringify(requestWith(app, definition))};`,
    `const MODULES = ${JSON.stringify(app.modules)};`,
    `const base = import.meta.env.BASE_URL + ${JSON.stringify(folder)};`,
    "export const sandbox = {",
    "  base,",
    `  trusted: { ${trusted.join(", ")} },`,
    "  modules: (module) => {",
    "    const found = MODULES[`${module.pack}@${module.version}/${module.path}`];",
    "    return found && { url: new URL(found.path, new URL(base, location.href)).href, integrity: found.integrity };",
    "  },",
    "};",
    "",
  );
  return lines.join("\n");
}

/**
 * The headers of a file under the folder: the frames have an opaque origin, so their files load cross-origin; their
 * documents are sandboxed even when opened alone (docs/running-components-in-the-sandbox.md, Q3).
 */
function headersFor(path: string, response: ServerResponse): void {
  response.setHeader("Access-Control-Allow-Origin", "*");
  if (path.endsWith(".html")) response.setHeader("Content-Security-Policy", "sandbox allow-scripts");
}

/** Every file under `dir`, by its `/`-separated path there. */
async function filesUnder(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries.filter((entry) => entry.isFile()).map((entry) => relative(dir, join(entry.parentPath, entry.name)).split(sep).join("/"));
}

/** Builds the frame documents into `out`, with the libraries the configuration offers; gives their library definition. */
async function buildSandbox(app: App, root: string, out: string): Promise<LibraryDefinition> {
  await rm(out, { recursive: true, force: true });
  // The libraries first: the frame documents' files go over them, with the runtimes' import map.
  const offers = app.config.libraries.length > 0;
  const imports = offers ? await buildLibraries(app.config.libraries, { root, out: join(out, "libs"), shared: RUNTIMES }) : {};
  const libraries = offers ? await readLibraries(join(out, "libs")) : [];
  await buildFrames({ out, imports, libraries });
  return libraryDefinition(out);
}

/**
 * The CompMD plugin.
 *
 * @example
 * // vite.config.ts
 * import { compmd } from "@compmd/vite";
 * export default defineConfig({ plugins: [react(), compmd()] });
 */
export function compmd(options: CompmdOptions = {}): Plugin {
  const folder = (options.folder ?? "compmd/").replace(/^\/+/, "").replace(/\/*$/, "/");
  let config: ResolvedConfig;
  let app: App | undefined;
  let frames = "";
  let built: Promise<LibraryDefinition> | undefined;
  let definition: LibraryDefinition | undefined;

  /** Reads the app again, and its sandbox once its libraries are known. */
  const load = async (): Promise<App> => {
    app = await loadApp(config.root);
    built ??= buildSandbox(app, config.root, frames);
    definition = await built;
    return app;
  };

  /** The file on disk a path under `folder` names, or `undefined` for none. */
  const fileOf = (path: string): string | undefined => {
    if (path.startsWith("packs/")) return app?.served.get(path);
    const file = join(frames, ...path.split("/"));
    return !path.split("/").includes("..") && existsSync(file) ? file : undefined;
  };

  return {
    name: "compmd",
    configResolved(resolved) {
      config = resolved;
      frames = join(resolved.cacheDir, "compmd-frames");
    },
    async buildStart() {
      try {
        const loaded = await load();
        for (const warning of loaded.warnings) this.warn(warning);
      } catch (error) {
        this.error(error instanceof Error ? error.message : String(error));
      }
    },
    resolveId(id) {
      return id === VIRTUAL ? RESOLVED : undefined;
    },
    async load(id) {
      if (id !== RESOLVED) return undefined;
      return moduleCode(app ?? (await load()), folder, definition);
    },
    configureServer(server) {
      const prefix = `${config.base.replace(/\/*$/, "/")}${folder}`.replace(/^\.\//, "/");
      const watched = [join(config.root, "compmd.config.json"), join(config.root, "package.json")];
      server.watcher.add(watched);
      server.watcher.on("change", (file) => {
        const inPack = app?.packs.some((pack) => file.startsWith(pack.dir + sep));
        if (!watched.includes(file) && !inPack) return;
        void load().then(
          () => {
            const module = server.moduleGraph.getModuleById(RESOLVED);
            if (module) server.moduleGraph.invalidateModule(module);
            server.ws.send({ type: "full-reload" });
          },
          (error: unknown) => {
            config.logger.error(`[compmd] ${error instanceof Error ? error.message : String(error)}`);
          },
        );
      });
      server.middlewares.use((request, response, next) => {
        const url = new URL(request.url ?? "/", "http://localhost");
        if (!url.pathname.startsWith(prefix)) {
          next();
          return;
        }
        let file: string | undefined;
        try {
          file = fileOf(decodeURIComponent(url.pathname.slice(prefix.length)));
        } catch {
          // A malformed escape names no file.
        }
        if (!file) {
          next();
          return;
        }
        void readFile(file).then((body) => {
          headersFor(file, response);
          response.setHeader("Content-Type", TYPES[extname(file)] ?? "application/octet-stream");
          response.end(body);
        }, next);
      });
    },
    configurePreviewServer(server) {
      const prefix = `${config.base.replace(/\/*$/, "/")}${folder}`.replace(/^\.\//, "/");
      server.middlewares.use((request, response, next) => {
        const path = new URL(request.url ?? "/", "http://localhost").pathname;
        if (path.startsWith(prefix)) headersFor(path, response);
        next();
      });
    },
    async generateBundle() {
      // A server build, such as a prerender's, has no page for frames.
      if (!app || this.environment.config.consumer === "server") return;
      for (const path of await filesUnder(frames)) this.emitFile({ type: "asset", fileName: `${folder}${path}`, source: await readFile(join(frames, path)) });
      for (const [path, file] of app.served) this.emitFile({ type: "asset", fileName: `${folder}${path}`, source: await readFile(file) });
    },
  };
}
