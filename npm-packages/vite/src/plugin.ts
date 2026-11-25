// The Vite plugin (docs/vite.md): reads the configuration and the packs at build start, writes the bundle
// (docs/hosting.md, decision 3) and gives the app `virtual:compmd`. Without `out`, the dev server serves the bundle under
// `compmd/` and the build writes it into the output; with `out`, it is written there, for an adapter to embed or serve.
// With registries the host fetches from, the dev and preview servers are its proxy. A page with a configuration of its
// own, in its folder, gets that one (docs/vite-per-page-config.md).

import { existsSync } from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { readdir, readFile, writeFile } from "node:fs/promises";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { respond, type Bundle } from "@compmd/host";
import { folderBundle } from "@compmd/host/node";
import { NpmRegistry, registryProxy } from "@compmd/registry";
import { toNodeListener } from "@compmd/registry/node";
import type { Connect, ModuleGraph, ModuleNode, Plugin, ResolvedConfig, Rollup, UserConfig } from "vite";
import { AppError, configOf, loadApp, requestWith, type App, type ServedModule } from "./app.ts";
import { addFrames, clearBundle, MANIFEST, manifestOf, servedModules, writePacksOf, type BundledFrames } from "./bundle.ts";
import type { ReadConfig } from "./config.ts";
import { configFolderOf, configFolders, type ConfigFolder } from "./configs.ts";
import { CONTRACT_HEADERS, headersFile, SERVED } from "./headers.ts";
import { messageOf } from "./message-of.ts";
import { serialized } from "./serial.ts";

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

/**
 * The id of the virtual module of the configuration in `folder`: the root's keeps `\0virtual:compmd`, so a project
 * with one configuration is as it was (docs/vite-per-page-config.md, decision 8).
 */
export const virtualIdOf = (folder: ConfigFolder): string => (folder === "" ? RESOLVED : `${RESOLVED}?config=${encodeURIComponent(folder)}`);

/** The configuration folder of a virtual module's id, or `undefined` for another id. */
export function folderOfVirtualId(id: string): ConfigFolder | undefined {
  if (id === RESOLVED) return "";
  if (!id.startsWith(`${RESOLVED}?`)) return undefined;
  return new URLSearchParams(id.slice(RESOLVED.length + 1)).get("config") ?? "";
}

/** A configuration's name in a message: "the root's configuration", or "playground/'s". */
const configName = (folder: ConfigFolder): string => (folder === "" ? "the root's configuration" : `${folder}/'s configuration`);

/**
 * Why one page reaching the virtual modules of two configurations stops the build (docs/vite-per-page-config.md,
 * decision 5), or `undefined` when it reaches one at most. `reached` holds each virtual module's id with the modules that
 * import it.
 */
export function twoConfigurations(page: string, reached: ReadonlyMap<string, readonly string[]>): string | undefined {
  if (reached.size < 2) return undefined;
  const lines = [...reached].map(([id, importers]) => `- ${configName(folderOfVirtualId(id) ?? "")}, imported by ${importers.join(", ")}`);
  return `${page} reaches two configurations, which one page never may (docs/vite-per-page-config.md):\n${lines.join("\n")}`;
}

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
  const proxy = 'const proxy = new NpmRegistry({ url: (origin ?? "").replace(/\\/*$/, "/") + "registry/" });';
  return [...(fetch === "host" ? [proxy] : []), `export const registry = { ${options.join(", ")} };`];
}

/** The code of the bundle's origin, as `moduleCode` says. */
function originCode(folder: string | undefined, upToRoot: string | undefined): string {
  if (folder === undefined) return "globalThis.__COMPMD__?.origin";
  // `@vite-ignore`: the bundle is written beside the build, not a file Vite resolves at build time.
  if (upToRoot !== undefined) return `new URL(/* @vite-ignore */ ${JSON.stringify(`${upToRoot}${folder}`)}, import.meta.url).href`;
  return `import.meta.env.BASE_URL + ${JSON.stringify(folder)}`;
}

/**
 * The way from a built module's file up to the build's root, `../` for Vite's `assets/`, when the base is relative
 * (`./`), so a page's folder doesn't change where the bundle is; `undefined` in development, or with an absolute base.
 */
export function upToRootOf(command: "build" | "serve", base: string, assetsDir: string): string | undefined {
  if (command !== "build" || !(base === "" || base.startsWith("."))) return undefined;
  return "../".repeat(assetsDir.split("/").filter((part) => part !== "" && part !== ".").length);
}

/**
 * The code of `virtual:compmd`: the processor's request, the sandbox's options with the trusted components the app's
 * own build compiles, and the registries' options. Its paths are relative to the bundle's origin (docs/hosting.md,
 * decision 9), which the sandbox puts them under: the app's `folder` for a web app, or, with no folder (`out`), the
 * one the host's adapter gives (`globalThis.__COMPMD__.origin`).
 *
 * `upToRoot` is the way from the module's own file up to the build's root (`../`), for a build with a relative base:
 * the origin is then found from the module's file, never from the page, since a page may be in a folder of its own
 * (`playground/`) while the bundle is at the root.
 */
export function moduleCode(app: App, folder: string | undefined, frames: BundledFrames, modules: Record<string, ServedModule>, upToRoot?: string): string {
  const bindings = new Set(app.trusted.map(({ binding }) => binding));
  const lines = [...bindings].filter((binding) => binding !== "plain").map((binding) => `import { ${binding} } from "@compmd/vite/${binding}";`);
  if (Object.keys(app.config.registries).length > 0) lines.push('import { NpmRegistry } from "@compmd/registry";');
  app.trusted.forEach(({ file }, i) => lines.push(`import c${String(i)} from ${JSON.stringify(file)};`));
  const trusted = app.trusted.map(({ name, binding }, i) => `${JSON.stringify(name)}: ${binding === "plain" ? `c${String(i)}` : `${binding}(c${String(i)})`}`);
  lines.push(
    `export const request = ${JSON.stringify(requestWith(app, frames.definition))};`,
    `const MODULES = ${JSON.stringify(modules)};`,
    `const origin = ${originCode(folder, upToRoot)};`,
    "export const sandbox = {",
    ...(folder === undefined ? [] : ["  origin,"]),
    `  frames: ${JSON.stringify(`frames/${frames.hash}/`)},`,
    `  trusted: { ${trusted.join(", ")} },`,
    "  modules: (module) => MODULES[`${module.pack}@${module.version}/${module.path}`],",
    // The origins the configuration lets documents reach without asking the reader, for their images (spec §15).
    ...(app.config.origins.length === 0 ? [] : [`  origins: { configured: ${JSON.stringify(app.config.origins)} },`]),
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
export function proxyOf({ registries, fetch }: Pick<ReadConfig, "registries" | "fetch">, path: string): Listener | undefined {
  if (fetch !== "host" || Object.keys(registries).length === 0) return undefined;
  const upstream = Object.fromEntries(Object.entries(registries).map(([scope, url]) => [scope, new NpmRegistry({ url })]));
  return toNodeListener(registryProxy({ registries: upstream, base: path }));
}

/**
 * What the one proxy of a project's configurations passes on to (docs/vite-per-page-config.md, decision 7): the
 * registries of those that fetch through the host, by scope.
 *
 * @throws AppError when two configurations send one scope to different registries.
 */
export function proxiedOf(configs: readonly { folder: string; config: Pick<ReadConfig, "registries" | "fetch"> }[]): Pick<ReadConfig, "registries" | "fetch"> {
  const named = new Map<string, { url: string; folder: string }>();
  for (const { folder, config } of configs) {
    for (const [scope, url] of Object.entries(config.registries)) {
      const earlier = named.get(scope);
      if (earlier && earlier.url !== url) {
        const where = (at: string) => (at === "" ? "the root's configuration" : `${at}/'s configuration`);
        throw new AppError(`${where(earlier.folder)} and ${where(folder)} send ${scope} to different registries (${earlier.url}, ${url}): name the same one`);
      }
      named.set(scope, { url, folder });
    }
  }
  const proxied = configs.filter(({ config }) => config.fetch === "host").flatMap(({ config }) => Object.entries(config.registries));
  return { registries: Object.fromEntries(proxied), fetch: "host" };
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

/** What Vite scans for dependencies when an app names no entries: every page, but those it ignores itself. */
const DEFAULT_ENTRIES = ["**/*.html", "!**/node_modules/**", "!**/__tests__/**", "!**/coverage/**"];

/**
 * Keeps the bundle written to `out` inside the app's folder (`src-tauri/compmd`) out of the dev server's view: it isn't
 * the app's own files, so it reloads no page when it is written, and its frame documents aren't scanned for the app's
 * dependencies. The app's own `optimizeDeps.entries`, when it has them, are left as they are.
 */
export function outsideTheApp(user: UserConfig, out: string | undefined): UserConfig | undefined {
  if (out === undefined) return undefined;
  const root = resolve(user.root ?? process.cwd());
  const folder = relative(root, resolve(root, out)).split(sep).join("/");
  if (folder === "" || folder.startsWith("..")) return undefined;
  return {
    server: { watch: { ignored: [`${resolve(root, out).split(sep).join("/")}/**`] } },
    ...(user.optimizeDeps?.entries === undefined ? { optimizeDeps: { entries: [...DEFAULT_ENTRIES, `!${folder}/**`] } } : {}),
  };
}

/** A configuration's app, the frames its pages load, and the built modules they run. */
interface Configured {
  app: App;
  frames: BundledFrames;
  modules: Record<string, ServedModule>;
}

/** The modules of `chunk` and of every chunk it imports, at once or when it runs: what its page reaches. */
function reachedModules(chunk: Rollup.OutputChunk, output: Rollup.OutputBundle): Set<string> {
  const modules = new Set<string>();
  const seen = new Set<string>();
  const walk = (at: Rollup.OutputChunk): void => {
    if (seen.has(at.fileName)) return;
    seen.add(at.fileName);
    for (const id of at.moduleIds) modules.add(id);
    for (const file of [...at.imports, ...at.dynamicImports]) {
      const next = output[file];
      if (next?.type === "chunk") walk(next);
    }
  };
  walk(chunk);
  return modules;
}

/** The modules a dev server's module reached through, up to those nothing imports: a page's own scripts. */
function pagesOf(module: ModuleNode | undefined): Set<string> {
  const pages = new Set<string>();
  const seen = new Set<ModuleNode>();
  const walk = (node: ModuleNode): void => {
    if (seen.has(node)) return;
    seen.add(node);
    if (node.importers.size === 0 && node.id !== null) pages.add(node.id);
    for (const importer of node.importers) walk(importer);
  };
  if (module) walk(module);
  return pages;
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
  /** The folders with a configuration, the root's first (docs/vite-per-page-config.md). */
  let folders: ConfigFolder[] = [];
  /** Each configuration's app, with its frames and the built modules they load, by its folder. */
  let configured = new Map<ConfigFolder, Configured>();
  /** Where the bundle is written: `out`, or Vite's cache, whence the servers serve it and the build copies it. */
  let bundle = "";
  /** The URL path of `folder`, `/compmd/`, which the servers serve. */
  let prefix = "";
  let proxy: Listener | undefined;
  /** The frames built so far, by the libraries they offer: built once each, as the bundle is first written. */
  const framesBy = new Map<string, BundledFrames>();
  /** The bundle as the dev server serves it, read when it is first asked for after it was written. */
  let served: Promise<Bundle> | undefined;
  const source = options.source === undefined ? {} : { source: options.source };

  /** What the folders under the root that hold no page's configuration are: the bundle and the build's output. */
  const skipped = (): string[] => [resolve(config.root, config.build.outDir), config.cacheDir, ...(options.out === undefined ? [] : [resolve(config.root, options.out)])];

  /** The loads so far, each after the one before, so two never write the bundle at once (audit B20). */
  let loads: Promise<unknown> = Promise.resolve();
  /** Reads the configurations and their apps again and writes their packs, after the loads before it. */
  const load = (): Promise<Map<ConfigFolder, Configured>> => {
    const next = loads.then(loadOnce, loadOnce);
    loads = next.catch(() => undefined);
    return next;
  };
  /** Reads each configuration's app and writes their packs once for all, after their frames, built once each. */
  const loadOnce = async (): Promise<Map<ConfigFolder, Configured>> => {
    folders = configFolders(config.root, skipped());
    const apps: App[] = [];
    for (const at of folders) apps.push(await loadApp(config.root, at === "" ? options.config : undefined, { folder: join(config.root, at) }));
    proxy = proxyOf(proxiedOf(apps.map((app, i) => ({ folder: folders[i] ?? "", config: app.config }))), `${prefix}registry/`);
    if (framesBy.size === 0) await clearBundle(bundle);
    const frames: BundledFrames[] = [];
    for (const app of apps) {
      const key = JSON.stringify(app.config.libraries);
      const built = framesBy.get(key) ?? (await addFrames(app, { root: config.root, out: bundle, ...source }));
      framesBy.set(key, built);
      frames.push(built);
    }
    const packs = await writePacksOf(apps, bundle);
    const [root, rootFrames] = [apps[0], frames[0]];
    // The manifest a host reads, with `out`: the root's configuration's policy, and every pack the bundle holds.
    if (root && rootFrames) await writeFile(join(bundle, MANIFEST), `${JSON.stringify({ ...manifestOf(root, rootFrames, options.source), packs: packs.get(root) ?? [] }, null, 2)}\n`);
    configured = new Map(apps.map((app, i) => [folders[i] ?? "", { app, frames: frames[i] as BundledFrames, modules: servedModules(app, packs.get(app) ?? []) }]));
    // Their packs may have changed: the dev server reads the bundle again.
    served = undefined;
    return configured;
  };
  /** The dev server's module graph, in development. */
  let graph: ModuleGraph | undefined;
  /** Why the virtual module `id` and another one loaded before it share a page, or `undefined` when they don't. */
  const sharedPage = (modules: ModuleGraph, id: string): string | undefined => {
    const own = pagesOf(modules.getModuleById(id));
    const importersOf = (at: string): string[] =>
      [...(modules.getModuleById(at)?.importers ?? [])].flatMap((importer) => (importer.id === null ? [] : [relative(config.root, importer.id)]));
    for (const other of modules.idToModuleMap.keys()) {
      if (other === id || folderOfVirtualId(other) === undefined) continue;
      const page = [...pagesOf(modules.getModuleById(other))].find((candidate) => own.has(candidate));
      if (page !== undefined) return twoConfigurations(relative(config.root, page), new Map([other, id].map((at) => [at, importersOf(at)])));
    }
    return undefined;
  };
  /** Whether `file` is in one of the configurations' packs. */
  const inPack = (file: string): boolean => [...configured.values()].some(({ app }) => app.packs.some((pack) => file.startsWith(pack.dir + sep)));

  return {
    name: "compmd",
    config: (user) => outsideTheApp(user, options.out),
    configResolved(resolved) {
      config = resolved;
      bundle = options.out === undefined ? join(resolved.cacheDir, "compmd-bundle") : resolve(resolved.root, options.out);
      prefix = `${resolved.base.replace(/\/*$/, "/")}${folder}`.replace(/^\.\//, "/");
    },
    async buildStart() {
      try {
        const loaded = await load();
        for (const [at, { app }] of loaded) for (const warning of app.warnings) this.warn(at === "" ? warning : `${at}/: ${warning}`);
      } catch (error) {
        this.error(messageOf(error));
      }
    },
    resolveId(id, importer) {
      if (id !== VIRTUAL) return undefined;
      // Before the first load, as a request without an importer may come: the folders as they are now.
      return virtualIdOf(configFolderOf(importer, config.root, folders.length > 0 ? folders : configFolders(config.root, skipped())));
    },
    async load(id) {
      const at = folderOfVirtualId(id);
      if (at === undefined) return undefined;
      // In development, a page reaching this configuration and another one loaded before (decision 5): the error
      // shows in the page. By the time a virtual module is loaded, the modules that import it are in the graph.
      const why = graph && sharedPage(graph, id);
      if (why) throw new AppError(why);
      const entry = configured.get(at) ?? (await load()).get(at);
      if (entry === undefined) throw new AppError(`${at}/ has no configuration any more`);
      return moduleCode(entry.app, options.out === undefined ? folder : undefined, entry.frames, entry.modules, upToRootOf(config.command, config.base, config.build.assetsDir));
    },
    configureServer(server) {
      server.watcher.add([join(config.root, "compmd.config.json"), join(config.root, "package.json")]);
      // Saves in a burst are one reload, never two at once, and a failed one is logged (audit B20).
      const reload = serialized(
        async () => {
          await load();
          for (const [id, module] of server.moduleGraph.idToModuleMap) if (folderOfVirtualId(id) !== undefined) server.moduleGraph.invalidateModule(module);
          server.ws.send({ type: "full-reload" });
        },
        {
          onError: (error) => {
            config.logger.error(`[compmd] ${messageOf(error)}`);
          },
        },
      );
      /** Whether `file` is a configuration, or a configuration folder's `package.json`, under the root (decision 11). */
      const isConfig = (file: string): boolean => {
        const at = relative(config.root, dirname(file)).split(sep).join("/");
        if (at.startsWith("..") || at.split("/").includes("node_modules") || skipped().some((dir) => file.startsWith(dir + sep))) return false;
        return basename(file) === "compmd.config.json" || (basename(file) === "package.json" && (at === "" || folders.includes(at)));
      };
      // A file added to a pack, or taken out of it, changes it as an edit does.
      const onFile = (file: string): void => {
        if (isConfig(file) || inPack(file)) reload();
      };
      graph = server.moduleGraph;
      for (const event of ["change", "add", "unlink"] as const) server.watcher.on(event, onFile);
      server.middlewares.use(proxying(`${prefix}registry/`, () => proxy));
      // With `out`, the adapter serves the bundle.
      if (options.out !== undefined) return;
      // The bundle under `prefix`, as every host serves it: the serving contract's `respond` (`@compmd/host`).
      server.middlewares.use((request, response, next) => {
        const url = new URL(request.url ?? "/", "http://localhost");
        if (!url.pathname.startsWith(prefix)) {
          next();
          return;
        }
        const asked = new Request(new URL(`/${url.pathname.slice(prefix.length)}${url.search}`, url), { method: request.method ?? "GET" });
        void (served ??= folderBundle(bundle))
          .then((held) => respond(asked, held))
          .then(async (answer) => {
            response.statusCode = answer.status;
            answer.headers.forEach((value, name) => {
              response.setHeader(name, value);
            });
            response.end(Buffer.from(await answer.arrayBuffer()));
          }, next);
      });
    },
    async configurePreviewServer(server) {
      // A preview builds nothing: the proxy is the configurations'.
      const configs = [];
      for (const at of configFolders(config.root, skipped())) configs.push({ folder: at, config: await configOf(join(config.root, at), at === "" ? options.config : undefined) });
      const previewed = proxyOf(proxiedOf(configs), `${prefix}registry/`);
      server.middlewares.use(proxying(`${prefix}registry/`, () => previewed));
      server.middlewares.use((request, response, next) => {
        const path = new URL(request.url ?? "/", "http://localhost").pathname;
        if (path.startsWith(prefix) && isServed(path.slice(prefix.length))) setContractHeaders(response);
        next();
      });
    },
    async generateBundle(_options, output) {
      // A server build, such as a prerender's, has no page for frames.
      if (configured.size === 0 || this.environment.config.consumer === "server") return;
      for (const chunk of Object.values(output)) {
        if (chunk.type !== "chunk" || !chunk.isEntry) continue;
        const reached = new Map<string, string[]>();
        for (const id of reachedModules(chunk, output)) {
          if (folderOfVirtualId(id) !== undefined) reached.set(id, [...(this.getModuleInfo(id)?.importers ?? [])].map((importer) => relative(config.root, importer)));
        }
        const why = twoConfigurations(relative(config.root, chunk.facadeModuleId ?? chunk.fileName), reached);
        if (why) this.error(why);
      }
      // With `out`, the bundle is there already.
      if (options.out !== undefined) return;
      for (const served of SERVED) {
        for (const path of await filesUnder(join(bundle, served))) this.emitFile({ type: "asset", fileName: `${folder}${served}${path}`, source: await readFile(join(bundle, served, path)) });
      }
      if (config.publicDir !== "" && existsSync(join(config.publicDir, "_headers"))) this.warn(`the app has its own _headers: add the bundle's to it, for a static host:\n${headersFile(prefix)}`);
      else this.emitFile({ type: "asset", fileName: "_headers", source: headersFile(prefix) });
    },
  };
}
