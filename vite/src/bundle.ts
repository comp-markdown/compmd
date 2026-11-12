// The bundle (docs/hosting.md, decision 3): the frames under their content hash, each pack's files by its place, and
// `manifest.json`, written into a folder a host serves or embeds. The plugin writes it for its own servers and output,
// or into its `out`; `buildBundle` writes it for a host built without Vite. Node only.

import { cp, mkdir, rename, rm, writeFile } from "node:fs/promises";
import { join, posix, relative, sep } from "node:path";
import type { Policy } from "@compmd/processor";
import { buildFrames, buildLibraries, compile, hashOf, readLibraries, RUNTIMES, type Compiled, type LibraryDefinition } from "@compmd/sandbox/build";
import semver from "semver";
import { AppError, loadApp, requestWith, sha384Of, type App, type ServedModule } from "./app.ts";
import { headersFile } from "./headers.ts";
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

/** What in a pack's folder isn't its files: its own dependencies, and hidden files and folders (`.git`). */
const isSkipped = (name: string): boolean => name === "node_modules" || name.startsWith(".");

/** The custom scheme's form `source` names, ending in `/`. */
export const sourceOf = (source: string): string => source.replace(/\/*$/, "/");

/**
 * Builds the frames into `out/frames/<hash>/`, with the libraries the configuration offers, and removes what an
 * earlier bundle left in `out`.
 */
export async function writeFrames(app: App, { root, out, source }: { root: string; out: string; source?: string }): Promise<BundledFrames> {
  await Promise.all(OWNED.map((name) => rm(join(out, name), { recursive: true, force: true })));
  const staging = join(out, ".staging");
  await mkdir(staging, { recursive: true });
  // The libraries first: the frame documents' files go over them, with the runtimes' import map.
  const offers = app.config.libraries.length > 0;
  const imports = offers ? await buildLibraries(app.config.libraries, { root, out: join(staging, "libs"), shared: RUNTIMES }) : {};
  const libraries = offers ? await readLibraries(join(staging, "libs")) : [];
  // The frames' path, not their hash's: the hash is of the files the policies are in.
  const definition = await buildFrames({ out: staging, imports, libraries, ...(source === undefined ? {} : { source: `${sourceOf(source)}frames/` }) });
  const hash = await hashOf(staging);
  await mkdir(join(out, "frames"), { recursive: true });
  await rename(staging, join(out, "frames", hash));
  return { hash, definition };
}

/** Whether the configuration denies `pack` at `version`: its files are never served, and its uses fall back. */
const isDenied = (app: App, pack: string, version: string): boolean => app.config.deny.some((denied) => denied.pack === pack && (denied.version === undefined || semver.satisfies(version, denied.version)));

/** Copies the files of the pack at `dir` into `to`, as published: all but its own dependencies and hidden files. */
const copyPack = (dir: string, to: string): Promise<void> => cp(dir, to, { recursive: true, filter: (file) => !relative(dir, file).split(sep).some(isSkipped) });

/**
 * The bundle's manifest: the frames, each pack the app offers (a denied pack too, so the processor says why it is
 * refused), the frames' libraries, and the processor's policy and ambient packs.
 */
export function manifestOf(app: App, frames: BundledFrames, source?: string): Manifest {
  const request = requestWith(app, frames.definition);
  const packs = app.packs
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
  return {
    format: 1,
    frames: { hash: frames.hash },
    ...(source === undefined ? {} : { source: sourceOf(source) }),
    packs,
    libraries: frames.definition.libraries.map(({ name, version }) => ({ name, version })),
    policy: request.policy,
    ambient: request.ambient,
  };
}

/** The languages a pack's source components are written in, which the bundle compiles. */
const SOURCES: ReadonlySet<ComponentFile["language"]> = new Set(["jsx", "tsx", "svelte", "ts"]);

/** The built module of the source component at `path`, beside it; a compiled Svelte component's styles are `.css`. */
export const builtPathOf = (path: string, kind: "js" | "css" = "js"): string => `${path}.${kind}`;

/**
 * Compiles `pack`'s source components into built modules beside them in `to`, as a frame's compiler would
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
      throw new AppError(`${pack.name}@${pack.version}: \`${path}\` doesn't compile: ${error instanceof Error ? error.message : String(error)}`);
    }
    const files: [string, string][] = [[builtPathOf(path), compiled.code], ...(compiled.css === undefined ? [] : [[builtPathOf(path, "css"), compiled.css] as [string, string]])];
    for (const [file, text] of files) {
      await writeFile(join(to, ...file.split("/")), text);
      written[file] = sha384Of(text);
    }
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
  await rm(join(out, "packs"), { recursive: true, force: true });
  const manifest = manifestOf(app, frames, source);
  for (const bundled of manifest.packs.filter(({ pack, version }) => !isDenied(app, pack, version))) {
    const pack = app.packs.find(({ name, version }) => name === bundled.pack && version === bundled.version);
    if (pack === undefined) continue;
    const to = join(out, "packs", `${pack.name}@${pack.version}`);
    await copyPack(pack.dir, to);
    if (bundled.trust !== "trusted") bundled.modules = { ...bundled.modules, ...(await compileSources(pack, to)) };
  }
  await writeFile(join(out, MANIFEST), `${JSON.stringify(manifest, null, 2)}\n`);
  return manifest;
}

/**
 * The built modules the frames load, by the place the processor names (`<pack>@<version>/<path>`), from the manifest:
 * a `.js` module by its own place, a compiled source component by its source's, with its styles.
 */
export function servedModules(app: App, manifest: Manifest): Record<string, ServedModule> {
  const served: Record<string, ServedModule> = {};
  for (const { pack, version, modules } of manifest.packs) {
    const components = app.packs.find((found) => found.name === pack && found.version === version)?.components ?? [];
    const sources = components.filter(({ language }) => SOURCES.has(language)).map(({ path }) => path);
    const at = (path: string) => `packs/${pack}@${version}/${path}`;
    for (const [path, integrity] of Object.entries(modules)) {
      if (path.endsWith(".css")) continue;
      const source = sources.find((candidate) => builtPathOf(candidate) === path);
      const css = source === undefined ? undefined : builtPathOf(source, "css");
      const styles = css === undefined || modules[css] === undefined ? {} : { css: { path: at(css), integrity: modules[css] } };
      served[`${pack}@${version}/${source ?? path}`] = { path: at(path), integrity, ...styles };
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
