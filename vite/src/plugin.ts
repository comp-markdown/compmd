// The Vite plugin (docs/vite.md): reads the configuration and the packs at build start, writes the bundle
// (docs/hosting.md, decision 3) and gives the app `virtual:compmd`. Without `out`, the dev server serves the bundle under
// `compmd/` and the build writes it into the output; with `out`, it is written there, for an adapter to embed or serve.
// With registries the host fetches from, the dev and preview servers are its proxy.

import { existsSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { readdir, readFile } from "node:fs/promises";
import { extname, join, relative, resolve, sep } from "node:path";
import { NpmRegistry, registryProxy } from "@compmd/registry";
import { toNodeListener } from "@compmd/registry/node";
import type { Connect, Plugin, ResolvedConfig } from "vite";
import { configOf, loadApp, requestWith, type App, type ServedModule } from "./app.ts";
import { servedModules, writeFrames, writePacks, type BundledFrames } from "./bundle.ts";
import type { ReadConfig } from "./config.ts";
import { CONTRACT_HEADERS, headersFile, SERVED } from "./headers.ts";

/** The plugin's options. */
export interface CompmdOptions {
  /** The folder of the app, under its base, where the bundle is served; `compmd/` by default. Not with `out`. */
  folder?: string;
  /**
   * The configuration, in place of `compmd.config.json`: for a tool that serves a page of its own, such as a pack's
   * test page. Folder packs in it are relative to Vite's root, as in the file.
   */
  config?: unknown;
  /**
   * The folder, relative to Vite's root, an adapter embeds or serves the bundle from, in place of the web output
   * (`src-tauri/compmd`): the dev server then serves no bundle, since the adapter does, and the page takes its origin
   * from the adapter (`globalThis.__COMPMD__`).
   */
  out?: string;
  /** The custom scheme's form the bundle is served at, `compmd://localhost/`, which the frames' policies name. */
  source?: string;
}

const VIRTUAL = "virtual:compmd";
const RESOLVED = `\0${VIRTUAL}`;
const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".map": "application/json", ".md": "text/markdown" };

/**
 * The code of `virtual:compmd`'s `registry`, after `origin`: what `resolvePacks` takes from the configuration. With
 * `fetch: "host"`, every scope's registry is the host's proxy, at `registry/` on the origin; without registries,
 * `undefined`.
 */
function registryCode({ registries, fetch, maxPackSize }: ReadConfig): string[] {
  const scopes = Object.entries(registries);
  if (scopes.length === 0) return ["export const registry = undefined;"];
  const entries = scopes.map(([scope, url]) => `${JSON.stringify(scope)}: ${fetch === "host" ? "proxy" : `new NpmRegistry({ url: ${JSON.stringify(url)} })`}`);
  const options = [`registries: { ${entries.join(", ")} }`, `fetch: ${JSON.stringify(fetch)}`, ...(maxPackSize === undefined ? [] : [`maxPackSize: ${String(maxPackSize)}`])];
  return [...(fetch === "host" ? ['const proxy = new NpmRegistry({ url: origin + "registry/" });'] : []), `export const registry = { ${options.join(", ")} };`];
}

/**
 * The code of `virtual:compmd`: the processor's request, the sandbox's options with the trusted components the app's
 * own build compiles, and the registries' options. Its paths are relative to the bundle's origin: the adapter's
 * (`globalThis.__COMPMD__.origin`), or else the app's `folder` (docs/hosting.md, decision 9).
 */
export function moduleCode(app: App, folder: string, frames: BundledFrames, modules: Record<string, ServedModule>): string {
  const bindings = new Set(app.trusted.map(({ binding }) => binding));
  const lines = [...bindings].filter((binding) => binding !== "plain").map((binding) => `import { ${binding} } from "@compmd/vite/${binding}";`);
  if (Object.keys(app.config.registries).length > 0) lines.push('import { NpmRegistry } from "@compmd/registry";');
  app.trusted.forEach(({ file }, i) => lines.push(`import c${String(i)} from ${JSON.stringify(file)};`));
  const trusted = app.trusted.map(({ name, binding }, i) => `${JSON.stringify(name)}: ${binding === "plain" ? `c${String(i)}` : `${binding}(c${String(i)})`}`);
  lines.push(
    `export const request = ${JSON.stringify(requestWith(app, frames.definition))};`,
    `const FRAMES = ${JSON.stringify(`frames/${frames.hash}/`)};`,
    `const MODULES = ${JSON.stringify(modules)};`,
    "const given = globalThis.__COMPMD__?.origin;",
    `const origin = given === undefined ? import.meta.env.BASE_URL + ${JSON.stringify(folder)} : given.replace(/\\/*$/, "/");`,
    "export const sandbox = {",
    "  base: origin + FRAMES,",
    `  trusted: { ${trusted.join(", ")} },`,
    "  modules: (module) => {",
    "    const found = MODULES[`${module.pack}@${module.version}/${module.path}`];",
    "    return found && { url: new URL(origin + found.path, location.href).href, integrity: found.integrity };",
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

/** Whether `path`, under the bundle's origin, is one the serving contract serves: in `frames/` or `packs/`. */
const isServed = (path: string): boolean => SERVED.some((folder) => path.startsWith(folder)) && !path.split("/").some((part) => part === "" || part === "." || part === "..");

/** Sets the serving contract's headers (docs/hosting.md, decision 8). */
function setContractHeaders(response: ServerResponse): void {
  for (const [name, value] of Object.entries(CONTRACT_HEADERS)) response.setHeader(name, value);
}

/** Every file under `dir`, by its `/`-separated path there; none when there is no `dir`. */
async function filesUnder(dir: string): Promise<string[]> {
  if (!existsSync(dir)) return [];
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries.filter((entry) => entry.isFile()).map((entry) => relative(dir, join(entry.parentPath, entry.name)).split(sep).join("/"));
}

/**
 * The CompMD plugin.
 *
 * @example
 * // vite.config.ts
 * import { compmd } from "@compmd/vite";
 * export default defineConfig({ plugins: [react(), compmd()] });
 * // a Tauri app's: the bundle in `src-tauri/compmd`, which the plugin embeds
 * export default defineConfig({ plugins: [react(), compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })] });
 */
export function compmd(options: CompmdOptions = {}): Plugin {
  const folder = (options.folder ?? "compmd/").replace(/^\/+/, "").replace(/\/*$/, "/");
  let config: ResolvedConfig;
  let app: App | undefined;
  /** Where the bundle is written: `out`, or Vite's cache, whence the servers serve it and the build copies it. */
  let bundle = "";
  /** The URL path of `folder`, `/compmd/`, which the servers serve. */
  let prefix = "";
  let proxy: Listener | undefined;
  let built: Promise<BundledFrames> | undefined;
  let frames: BundledFrames | undefined;
  /** The built modules the frames load, from the bundle's manifest. */
  let modules: Record<string, ServedModule> = {};
  const source = options.source === undefined ? {} : { source: options.source };

  /** Reads the app again and writes its packs, after its frames, which are built once. */
  const load = async (): Promise<App> => {
    const loaded = await loadApp(config.root, options.config);
    proxy = proxyOf(loaded.config, `${prefix}registry/`);
    built ??= writeFrames(loaded, { root: config.root, out: bundle, ...source });
    frames = await built;
    modules = servedModules(loaded, await writePacks(loaded, bundle, frames, options.source));
    app = loaded;
    return loaded;
  };

  /** The file of the bundle `path` names, or `undefined` for none the contract serves. */
  const fileOf = (path: string): string | undefined => {
    const file = join(bundle, ...path.split("/"));
    return isServed(path) && existsSync(file) ? file : undefined;
  };

  return {
    name: "compmd",
    configResolved(resolved) {
      config = resolved;
      bundle = options.out === undefined ? join(resolved.cacheDir, "compmd-bundle") : resolve(resolved.root, options.out);
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
      const loaded = app ?? (await load());
      return moduleCode(loaded, folder, frames ?? (await (built as Promise<BundledFrames>)), modules);
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
      // With `out`, the adapter serves the bundle.
      if (options.out !== undefined) return;
      server.middlewares.use((request, response, next) => {
        const url = new URL(request.url ?? "/", "http://localhost");
        let file: string | undefined;
        try {
          file = url.pathname.startsWith(prefix) ? fileOf(decodeURIComponent(url.pathname.slice(prefix.length))) : undefined;
        } catch {
          // A malformed escape names no file.
        }
        if (!file) {
          next();
          return;
        }
        void readFile(file).then((body) => {
          setContractHeaders(response);
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
        if (path.startsWith(prefix) && isServed(path.slice(prefix.length))) setContractHeaders(response);
        next();
      });
    },
    async generateBundle() {
      // A server build, such as a prerender's, has no page for frames; with `out`, the bundle is there already.
      if (!app || options.out !== undefined || this.environment.config.consumer === "server") return;
      for (const served of SERVED) {
        for (const path of await filesUnder(join(bundle, served))) this.emitFile({ type: "asset", fileName: `${folder}${served}${path}`, source: await readFile(join(bundle, served, path)) });
      }
      if (config.publicDir !== "" && existsSync(join(config.publicDir, "_headers"))) this.warn(`the app has its own _headers: add the bundle's to it, for a static host:\n${headersFile(prefix)}`);
      else this.emitFile({ type: "asset", fileName: "_headers", source: headersFile(prefix) });
    },
  };
}
