// The Vite plugin (docs/vite.md): reads the configuration and the packs at build start, gives the app `virtual:compmd`,
// builds the sandbox's frame documents and libraries, serves them and the packs' built modules under `compmd/` in dev,
// and writes them into the build. With registries the host fetches from, the dev and preview servers are its proxy.

import { existsSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { readdir, readFile, rm } from "node:fs/promises";
import { extname, join, relative, sep } from "node:path";
import { NpmRegistry, registryProxy } from "@compmd/registry";
import { toNodeListener } from "@compmd/registry/node";
import { buildFrames, buildLibraries, readLibraries, RUNTIMES, type LibraryDefinition } from "@compmd/sandbox/build";
import type { Connect, Plugin, ResolvedConfig } from "vite";
import { configOf, loadApp, type App } from "./app.ts";
import type { ReadConfig } from "./config.ts";

/** The plugin's options. */
export interface CompmdOptions {
  /** The folder of the app, under its base, where the sandbox and the packs' files are served; `compmd/` by default. */
  folder?: string;
  /**
   * The configuration, in place of `compmd.config.json`: for a tool that serves a page of its own, such as a pack's
   * test page. Folder packs in it are relative to Vite's root, as in the file.
   */
  config?: unknown;
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
 * The code of `virtual:compmd`'s `registry`, after `base`: what `resolvePacks` takes from the configuration. With
 * `fetch: "host"`, every scope's registry is the app's proxy, under `base`; without registries, `undefined`.
 */
function registryCode({ registries, fetch, maxPackSize }: ReadConfig): string[] {
  const scopes = Object.entries(registries);
  if (scopes.length === 0) return ["export const registry = undefined;"];
  const entries = scopes.map(([scope, url]) => `${JSON.stringify(scope)}: ${fetch === "host" ? "proxy" : `new NpmRegistry({ url: ${JSON.stringify(url)} })`}`);
  const options = [`registries: { ${entries.join(", ")} }`, `fetch: ${JSON.stringify(fetch)}`, ...(maxPackSize === undefined ? [] : [`maxPackSize: ${String(maxPackSize)}`])];
  return [...(fetch === "host" ? ['const proxy = new NpmRegistry({ url: base + "registry/" });'] : []), `export const registry = { ${options.join(", ")} };`];
}

/**
 * The code of `virtual:compmd`: the processor's request, the sandbox's options with the trusted components the app's
 * own build compiles, and the registries' options; with the frames' `definition`, the request's policy says what they
 * provide.
 */
export function moduleCode(app: App, folder: string, definition?: LibraryDefinition): string {
  const bindings = new Set(app.trusted.map(({ binding }) => binding));
  const lines = [...bindings].filter((binding) => binding !== "plain").map((binding) => `import { ${binding} } from "@compmd/vite/${binding}";`);
  if (Object.keys(app.config.registries).length > 0) lines.push('import { NpmRegistry } from "@compmd/registry";');
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
    ...registryCode(app.config),
    "",
  );
  return lines.join("\n");
}

/** A Node request listener, or Connect's. */
type Listener = (request: IncomingMessage, response: ServerResponse) => void;

/**
 * The app's proxy for its registries (docs/packs-from-a-registry.md, R1, F1), served at `path`: `undefined` unless the
 * configuration names registries and the host fetches from them.
 */
export function proxyOf({ registries, fetch }: ReadConfig, path: string): Listener | undefined {
  if (fetch !== "host" || Object.keys(registries).length === 0) return undefined;
  const upstream = Object.fromEntries(Object.entries(registries).map(([scope, url]) => [scope, new NpmRegistry({ url })]));
  return toNodeListener(registryProxy({ registries: upstream, base: path }));
}

/** A middleware that hands what is under `path` to the proxy `proxy()` gives, when there is one. */
function proxying(path: string, proxy: () => Listener | undefined): Connect.NextHandleFunction {
  return (request, response, next) => {
    const listener = proxy();
    if (listener && new URL(request.url ?? "/", "http://localhost").pathname.startsWith(path)) listener(request, response);
    else next();
  };
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
  return buildFrames({ out, imports, libraries });
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
  /** The URL path of `folder`, `/compmd/`, which the servers serve. */
  let prefix = "";
  let proxy: Listener | undefined;
  let built: Promise<LibraryDefinition> | undefined;
  let definition: LibraryDefinition | undefined;

  /** Reads the app again, and its sandbox once its libraries are known. */
  const load = async (): Promise<App> => {
    app = await loadApp(config.root, options.config);
    proxy = proxyOf(app.config, `${prefix}registry/`);
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
      prefix = `${resolved.base.replace(/\/*$/, "/")}${folder}`.replace(/^\.\//, "/");
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
      server.middlewares.use(proxying(`${prefix}registry/`, () => proxy));
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
    async configurePreviewServer(server) {
      // A preview builds nothing: the proxy is the configuration's.
      const previewed = proxyOf(await configOf(config.root, options.config), `${prefix}registry/`);
      server.middlewares.use(proxying(`${prefix}registry/`, () => previewed));
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
