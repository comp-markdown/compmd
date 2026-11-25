// The bundle (docs/hosting.md, decision 3): the frames under their content hash, each pack's files by its place, and
// `manifest.json`, written into a folder a host serves or embeds. The plugin writes it for its own servers and output,
// or into its `out`; `buildBundle` writes it for a host built without Vite. Node only.

import { existsSync } from "node:fs";
import { cp, mkdir, readdir, rename, rm, writeFile } from "node:fs/promises";
import { dirname, join, posix, resolve, sep } from "node:path";
import { newestMatching, type Policy } from "@compmd/processor";
import { buildFrames, buildLibraries, compile, hashOf, readLibraries, RUNTIMES, type Compiled, type LibraryDefinition } from "@compmd/sandbox/build";
import { AppError, loadApp, requestWith, sha384Of, type App, type ServedModule } from "./app.ts";
import { headersFile } from "./headers.ts";
import { messageOf } from "./message-of.ts";
import { isSkipped, PackError, publishedFiles } from "./pack-folder.ts";
import type { ComponentFile, Pack } from "./packs.ts";

/** The manifest's file in the bundle. */
export const MANIFEST = "manifest.json";

/** A pack the bundle holds, under `packs/<pack>@<version>/`. */
export interface BundledPack {
  pack: string;
  version: string;
  /** The integrity the registry published for it, from the app's lock file. */
  integrity?: string;
  /** The trust the configuration gives it. */
  trust: "strict" | "permissive" | "trusted";
  /** Its built modules, by their path in the pack, with their `sha384`, which a frame checks. */
  modules: Record<string, string>;
}

/** `manifest.json`: the bundle's one description, which every host reads, in JavaScript and in Rust (`compmd-host`). */
export interface Manifest {
  format: 1;
  /** The frames' build, served at `frames/<hash>/`. */
  frames: { hash: string };
  /** The custom scheme's form the frames' policies name (`compmd://localhost/`); none for http and https origins. */
  source?: string;
  packs: BundledPack[];
  /** The libraries the frames offer component code, besides the runtimes. */
  libraries: { name: string; version: string }[];
  /** The processor's policy (`policyOf`), with what the frames provide. */
  policy: Policy;
  /** The packs every document gets without importing them. */
  ambient: string[];
}

/** The frames a bundle holds: their hash, and their library definition. */
export interface BundledFrames {
  hash: string;
  definition: LibraryDefinition;
}

/** What the bundle owns in its folder: everything else there is left alone. */
const OWNED = ["frames", "packs", MANIFEST, "_headers", ".staging"];

/** The custom scheme's form `source` names, ending in `/`. */
export const sourceOf = (source: string): string => source.replace(/\/*$/, "/");

/**
 * Builds the frames into `out/frames/<hash>/`, with the libraries the configuration offers, and removes what an
 * earlier bundle left in `out`.
 */
export async function writeFrames(app: App, options: { root: string; out: string; source?: string }): Promise<BundledFrames> {
  await clearBundle(options.out);
  return addFrames(app, options);
}

/** Removes what an earlier bundle left in `out`: only what the bundle owns there. */
export async function clearBundle(out: string): Promise<void> {
  await Promise.all(OWNED.map((name) => rm(join(out, name), { recursive: true, force: true })));
}

/**
 * Builds the frames into `out/frames/<hash>/` beside those already there, with the libraries `app`'s configuration
 * offers: a bundle of pages with configurations of their own has one build for each distinct `libraries`
 * (docs/vite-per-page-config.md, decision 6).
 */
export async function addFrames(app: App, { root, out, source }: { root: string; out: string; source?: string }): Promise<BundledFrames> {
  const staging = join(out, ".staging");
  await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });
  // The libraries first: the frame documents' files go over them, with the runtimes' import map.
  const offers = app.config.libraries.length > 0;
  const imports = offers ? await buildLibraries(app.config.libraries, { root, out: join(staging, "libs"), shared: RUNTIMES }) : {};
  const libraries = offers ? await readLibraries(join(staging, "libs")) : [];
  // The frames' path, not their hash's: the hash is of the files the policies are in.
  const named = source === undefined ? {} : { source: `${sourceOf(source)}frames/`, packs: `${sourceOf(source)}packs/` };
  const definition = await buildFrames({ out: staging, imports, libraries, ...named });
  const hash = await hashOf(staging);
  await mkdir(join(out, "frames"), { recursive: true });
  // Two configurations with the same libraries build the same frames: the first is kept.
  if (existsSync(join(out, "frames", hash))) await rm(staging, { recursive: true, force: true });
  else await rename(staging, join(out, "frames", hash));
  return { hash, definition };
}

/**
 * Whether the configuration denies `pack` at `version`: its files are never served, and its uses fall back. A range
 * is read as the processor reads it (`newestMatching`), so what the bundle leaves out is what the processor refuses.
 */
const isDenied = (app: App, pack: string, version: string): boolean => app.config.deny.some((denied) => denied.pack === pack && (denied.version === undefined || newestMatching(denied.version, [version]) === version));

/** Copies the folder `dir` into `to`: all but its own dependencies, hidden files, and `bundle` when it is inside. */
async function copyFolder(dir: string, to: string, bundle: string): Promise<void> {
  await mkdir(to, { recursive: true });
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const from = join(dir, entry.name);
    // The bundle itself, when it is written inside the pack: an app whose own folder is a pack (`"./"`).
    if (isSkipped(entry.name) || from === bundle) continue;
    if (entry.isDirectory()) await copyFolder(from, join(to, entry.name), bundle);
    else await cp(from, join(to, entry.name));
  }
}

/**
 * What a pack publishes, by its paths in the pack: what its `package.json`'s `files` matches inside the pack, what npm
 * always adds, and every file its documents name. Without `files`, a folder pack publishes what its documents name,
 * and an installed one is all of its folder, as npm published it (`undefined`).
 *
 * @throws PackError when a folder pack without `files` names a path no one can list before it is used.
 */
async function publishedOf(pack: Pack): Promise<string[] | undefined> {
  const { files } = JSON.parse(pack.files["package.json"] ?? "{}") as { files?: unknown };
  const named = [...Object.keys(pack.files), ...pack.assets];
  const listed = await publishedFiles(pack.dir, files);
  if (listed === undefined && pack.folder === undefined) return undefined;
  if (listed === undefined && pack.unlisted.length > 0) {
    const paths = pack.unlisted.map((path) => `\`${path}\``).join(", ");
    throw new PackError(`${pack.name}: ${paths} can't be known until the pack is used, so its package.json's \`files\` must list what it may name, such as \`"icons/"\``);
  }
  return [...new Set([...(listed ?? (await publishedFiles(pack.dir, [])) ?? []), ...named])].sort();
}

/** Copies the pack's files into `to`, as published (`publishedOf`), leaving out `bundle` when it is inside. */
async function copyPack(pack: Pack, to: string, bundle: string): Promise<void> {
  const published = await publishedOf(pack);
  if (published === undefined) return copyFolder(pack.dir, to, bundle);
  for (const path of published) {
    const [from, into] = [join(pack.dir, ...path.split("/")), join(to, ...path.split("/"))];
    // A font's file or an image a document names that isn't there: the processor and the renderer say so.
    if (!existsSync(from) || resolve(from).startsWith(`${bundle}${sep}`)) continue;
    await mkdir(dirname(into), { recursive: true });
    await cp(from, into);
  }
}

/**
 * The bundle's manifest: the frames, each pack the app offers (a denied pack too, so the processor says why it is
 * refused), the frames' libraries, and the processor's policy and ambient packs.
 */
export function manifestOf(app: App, frames: BundledFrames, source?: string): Manifest {
  const request = requestWith(app, frames.definition);
  return {
    format: 1,
    frames: { hash: frames.hash },
    ...(source === undefined ? {} : { source: sourceOf(source) }),
    packs: bundledPacksOf(app),
    libraries: frames.definition.libraries.map(({ name, version }) => ({ name, version })),
    policy: request.policy,
    ambient: request.ambient,
  };
}

/** The packs `app` offers, a denied pack too, each with its trust and its built modules (`.js`). */
function bundledPacksOf(app: App): BundledPack[] {
  return app.packs
    .filter((pack) => !(pack.name in (app.request.policy.refused ?? {})))
    .map((pack): BundledPack => {
      const prefix = `${pack.name}@${pack.version}/`;
      const modules = Object.entries(app.modules).filter(([place]) => place.startsWith(prefix));
      return {
        pack: pack.name,
        version: pack.version,
        ...(pack.integrity === undefined ? {} : { integrity: pack.integrity }),
        trust: app.request.policy.packs?.[pack.name]?.level ?? "strict",
        modules: Object.fromEntries(modules.map(([place, { integrity }]) => [place.slice(prefix.length), integrity])),
      };
    });
}

/** The languages a pack's source components are written in, which the bundle compiles. */
const SOURCES: ReadonlySet<ComponentFile["language"]> = new Set(["jsx", "tsx", "svelte", "ts"]);

/** The built module of the source component at `path`, beside it; a compiled Svelte component's styles are `.css`. */
export const builtPathOf = (path: string): string => `${path}.js`;

/**
 * A compiled component as one module: its code, with a Svelte component's styles as its `css` export, which the frame
 * adopts, so one file and one hash hold both (a strict frame fetches nothing it could load the styles with).
 */
const moduleOf = ({ code, css }: Compiled): string => (css === undefined ? code : `${code}\nexport const css = ${JSON.stringify(css)};\n`);

/**
 * Compiles `pack`'s source components into modules beside them in `to`, `<path>.js`, as a frame's compiler would
 * (`@compmd/sandbox`'s `compile`), so no reader compiles them (docs/hosting.md, decision 3).
 *
 * @returns Each file written, by its path in the pack, with its `sha384`.
 * @throws AppError when a component doesn't compile.
 */
async function compileSources(pack: Pack, to: string): Promise<Record<string, string>> {
  const written: Record<string, string> = {};
  for (const { path, language } of pack.components.filter((component) => SOURCES.has(component.language))) {
    let compiled: Compiled;
    try {
      compiled = await compile({ name: posix.basename(path).replace(/\.[^.]+$/, ""), language, code: pack.files[path] ?? "" });
    } catch (error) {
      throw new AppError(`${pack.name}@${pack.version}: \`${path}\` doesn't compile: ${messageOf(error)}`);
    }
    const text = moduleOf(compiled);
    await writeFile(join(to, ...builtPathOf(path).split("/")), text);
    written[builtPathOf(path)] = sha384Of(text);
  }
  return written;
}

/**
 * Writes the packs' files into `out/packs/<pack>@<version>/` (all but denied ones), with their source components
 * compiled beside them (not a trusted pack's, which the app's own bundler compiles), and the manifest into
 * `out/manifest.json`, over what was there.
 *
 * @throws AppError when a pack's component doesn't compile.
 */
export async function writePacks(app: App, out: string, frames: BundledFrames, source?: string): Promise<Manifest> {
  const manifest = { ...manifestOf(app, frames, source), packs: (await writePacksOf([app], out)).get(app) ?? [] };
  await writeFile(join(out, MANIFEST), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

/**
 * Writes the packs of the apps of every configuration a Vite project has into `out/packs/<pack>@<version>/`, each
 * once (docs/vite-per-page-config.md, decision 6): all but those every configuration denies, with their source
 * components compiled beside them when a configuration doesn't trust the pack, since its pages run them in frames.
 *
 * @returns Each app's packs as its manifest lists them, with the modules its frames load.
 * @throws AppError when a pack's component doesn't compile.
 */
export async function writePacksOf(apps: readonly App[], out: string): Promise<Map<App, BundledPack[]>> {
  await rm(join(out, "packs"), { recursive: true, force: true });
  const bundledOf = new Map(apps.map((app) => [app, bundledPacksOf(app)]));
  /** Each pack's compiled source components, by its place, once compiled for a configuration that doesn't trust it. */
  const compiled = new Map<string, Record<string, string>>();
  const written = new Set<string>();
  for (const app of apps) {
    for (const bundled of (bundledOf.get(app) ?? []).filter(({ pack, version }) => !isDenied(app, pack, version))) {
      const pack = app.packs.find(({ name, version }) => name === bundled.pack && version === bundled.version);
      if (pack === undefined) continue;
      const place = `${pack.name}@${pack.version}`;
      const to = join(out, "packs", place);
      if (!written.has(place)) {
        await copyPack(pack, to, resolve(out));
        written.add(place);
      }
      if (bundled.trust === "trusted") continue;
      if (!compiled.has(place)) compiled.set(place, await compileSources(pack, to));
      bundled.modules = { ...bundled.modules, ...compiled.get(place) };
    }
  }
  return bundledOf;
}

/**
 * The built modules the frames load, by the place the processor names (`<pack>@<version>/<path>`), from the packs an
 * app's manifest lists: a `.js` module by its own place, a compiled source component by its source's (the processor's
 * `place`).
 */
export function servedModules(app: App, packs: readonly BundledPack[]): Record<string, ServedModule> {
  const served: Record<string, ServedModule> = {};
  for (const { pack, version, modules } of packs) {
    const components = app.packs.find((found) => found.name === pack && found.version === version)?.components ?? [];
    const sources = components.filter(({ language }) => SOURCES.has(language)).map(({ path }) => path);
    for (const [path, integrity] of Object.entries(modules)) {
      const source = sources.find((candidate) => builtPathOf(candidate) === path);
      served[`${pack}@${version}/${source ?? path}`] = { path: `packs/${pack}@${version}/${path}`, integrity };
    }
  }
  return served;
}

/** `buildBundle`'s options. */
export interface BundleOptions {
  /** The folder to write the bundle into; only `frames/`, `packs/`, `manifest.json` and `_headers` there are its. */
  out: string;
  /** The app's folder, with its `package.json` and `node_modules`: the working directory by default. */
  root?: string;
  /** The configuration, in place of `compmd.config.json` at `root`. */
  config?: unknown;
  /** The custom scheme's form the bundle is served at, `compmd://localhost/`; none for http and https origins. */
  source?: string;
}

/** What `buildBundle` wrote, and what the checks found that doesn't stop it. */
export interface BuiltBundle {
  manifest: Manifest;
  warnings: string[];
}

/**
 * Writes the bundle for a host built without Vite: the frames, the packs, `manifest.json`, and a `_headers` file for
 * a static host that reads one, with the serving contract's headers.
 *
 * @throws AppError, ConfigError or PackError, saying what to fix, as the plugin does.
 * @example
 * const { manifest } = await buildBundle({ out: "src-tauri/compmd", source: "compmd://localhost/" });
 * // src-tauri/compmd/frames/<manifest.frames.hash>/strict.html, src-tauri/compmd/packs/@acme/charts@1.4.0/…
 */
export async function buildBundle(options: BundleOptions): Promise<BuiltBundle> {
  const root = options.root ?? process.cwd();
  const app = await loadApp(root, options.config);
  const frames = await writeFrames(app, { root, out: options.out, ...(options.source === undefined ? {} : { source: options.source }) });
  const manifest = await writePacks(app, options.out, frames, options.source);
  await writeFile(join(options.out, "_headers"), headersFile("/"));
  return { manifest, warnings: app.warnings };
}
