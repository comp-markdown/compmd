// Finding the app's packs and reading each as the processor takes it (docs/vite.md, "What the plugin does"): every
// installed dependency that is a pack, and the folder packs the configuration lists; each pack's documents and the files
// they import, its integrity from the app's lock file, and its size.

import { readFile, realpath } from "node:fs/promises";
import { join, posix } from "node:path";
import { folderPacks, installedPacks, integrities as integritiesOf, lockFileOf as lockFileOfProject, type FoundPack, type PacksFound } from "@compmd/host/project";
import type { PackRelease } from "@compmd/processor";
import { nodeFiles } from "./node-files.ts";
import { documentOf, identityOf, joinedInPack, manifestAt, PackError, reachOf } from "./pack-folder.ts";

export { PackError };

/** A component file a pack's document imports, by its path in the pack. */
export interface ComponentFile {
  path: string;
  /** Its language, from its extension. */
  language: "jsx" | "tsx" | "svelte" | "js" | "ts";
}

/** A pack, found and read. */
export interface Pack {
  /** Its `package.json`'s name. */
  name: string;
  version: string;
  /** Where its files are. */
  dir: string;
  /** The configuration's key, for a folder pack. */
  folder?: string;
  /** Its document's path in the pack: the `compmd` field of `package.json`, or `index.comp.md`. */
  document: string;
  /** Every file its documents need as text, by path in the pack, with its text: documents, imports and data. */
  files: Record<string, string>;
  /** The other files its documents name, which the bundle serves as they are: fonts' files, definitions' images and links. */
  assets: string[];
  /** The paths its definitions or data write that no one can list before the pack is used (`./icons/{{icon}}.svg`). */
  unlisted: string[];
  /** The component files its documents import: built modules (`.js`) and source. */
  components: ComponentFile[];
  /** The integrity the registry published for it, from the app's lock file. */
  integrity?: string;
  /** The size of its files, in bytes. */
  size: number;
}

const LANGUAGES: Record<string, ComponentFile["language"]> = { ".jsx": "jsx", ".tsx": "tsx", ".svelte": "svelte", ".js": "js", ".ts": "ts" };

/** The language of a component file by its extension, or `undefined` for a file that isn't one. */
const languageOf = (path: string): ComponentFile["language"] | undefined => LANGUAGES[posix.extname(path)];

/**
 * Reads the pack at `dir`: its manifest, its document, and every file its documents name, recursively; the text of
 * its documents, imports and data. `dependency` is the name the app installed it by, for a package in `node_modules`.
 *
 * @throws PackError when `dir` has no `package.json` with a pack's name and a semver version, its name isn't
 *   `dependency`, a document names a path outside the pack, or one it imports or reads isn't in the pack.
 */
export async function readPack(dir: string, dependency?: string): Promise<Pack> {
  const manifest = await manifestAt(dir);
  if (!manifest) throw new PackError(`${dir} isn't a pack: it has no package.json with a name`);
  const { name, version } = identityOf(manifest, dir);
  if (dependency !== undefined && name !== dependency) throw new PackError(`${dependency} is installed, but its package.json names it \`${name}\`: a pack is served by its own name`);
  const document = documentOf(manifest);
  const read = (path: string) => readFile(join(dir, ...path.split("/")), "utf8").catch(() => undefined);
  const reach = await reachOf(document, read);
  const [outside] = reach.outside;
  if (outside !== undefined) throw new PackError(`${name}: \`${outside}\` leaves the pack, or isn't a path in it`);
  const files: Record<string, string> = { "package.json": await readFile(join(dir, "package.json"), "utf8") };
  for (const path of [document, ...reach.documents.keys(), ...reach.imports, ...reach.data]) {
    if (path in files) continue;
    const text = reach.documents.get(path) ?? (await read(path));
    if (text === undefined) throw new PackError(`${name}: \`${path}\` isn't in the pack${path === document ? ", which needs its document (package.json's `compmd`, or index.comp.md)" : ""}`);
    files[path] = text;
  }
  const components = Object.keys(files).flatMap((path): ComponentFile[] => {
    const language = languageOf(path);
    return language ? [{ path, language }] : [];
  });
  const size = Object.values(files).reduce((total, text) => total + Buffer.byteLength(text), 0);
  const assets = [...new Set([...reach.fonts, ...reach.destinations])].filter((path) => !(path in files)).sort();
  return { name, version, dir, document, files, assets, unlisted: reach.unlisted, components, size };
}

/** The packs `found` gives. @throws PackError with why they couldn't be found. */
function packsOf(found: PacksFound): FoundPack[] {
  if ("error" in found) throw new PackError(found.error);
  return found.packs;
}

/**
 * Finds the app's packs, as `compmd-project` finds them: every scoped dependency in `node_modules` that is a pack
 * (docs/vite.md, decision 3), and the folder packs `folders` names, relative to `base`: the folder of the
 * configuration that lists them, `root` by default. Each is then read here, for the bundle.
 *
 * @throws PackError when a folder isn't a pack, or an installed pack isn't one by its name.
 */
export async function findPacks(root: string, folders: readonly string[], base = root): Promise<Pack[]> {
  const packs: Pack[] = [];
  for (const { name, dir } of packsOf(installedPacks(root, nodeFiles))) packs.push(await readPack(await realpath(dir), name));
  for (const { dir, key } of packsOf(folderPacks([...folders], base, nodeFiles))) packs.push({ ...(await readPack(dir)), ...(key === undefined ? {} : { folder: key }) });
  return packs;
}

/** The lock file nearest above `root`, `pnpm-lock.yaml` or `package-lock.json`: a workspace keeps one for all its packages. */
export const lockFileOf = (root: string): string | undefined => lockFileOfProject(root, nodeFiles);

/**
 * The integrity the registry published for each package the app's lock file holds, by `name@version`: from
 * `pnpm-lock.yaml` or `package-lock.json`, the nearest above `root` (docs/vite.md, V1), as `compmd-project` reads them.
 * Empty without a lock file.
 */
export const integrities = (root: string): Map<string, string> => integritiesOf(root, nodeFiles);

/** The names of an import or an export entry: `[name, as]`. */
export function namesOf(entries: unknown): [string, string][] {
  if (!Array.isArray(entries)) return [];
  return entries.flatMap((entry: unknown): [string, string][] => {
    if (typeof entry === "string") return [[entry, entry]];
    if (typeof entry === "object" && entry !== null) {
      const { name, as } = entry as { name?: unknown; as?: unknown };
      if (typeof name === "string") return [[name, typeof as === "string" ? as : name]];
    }
    return [];
  });
}

/** The component files a pack's document imports, by the local name it gives each; `md` is its frontmatter's `md` key. */
export function localComponents(pack: Pack, md: Record<string, unknown>): Map<string, ComponentFile> {
  const local = new Map<string, ComponentFile>();
  for (const entry of Array.isArray(md.import) ? (md.import as unknown[]) : []) {
    const { from, names } = (typeof entry === "object" && entry !== null ? entry : {}) as { from?: unknown; names?: unknown };
    if (typeof from !== "string") continue;
    const path = joinedInPack(pack.document, from);
    const file = pack.components.find((component) => component.path === path);
    if (!file) continue;
    const stem = posix.basename(file.path).replace(/\.[^.]+$/, "");
    for (const [, as] of names === "*" ? [[stem, stem] as [string, string]] : namesOf(names)) local.set(as, file);
  }
  return local;
}

/**
 * A pack as the processor's release (`Request.packs`): its files, but empty text for the component files the app's
 * own bundle or the frames load (built modules, trusted packs' components), which the processor only needs to exist
 * (docs/vite.md, V3).
 */
export function releaseOf(pack: Pack, options: { trusted: boolean }): PackRelease {
  const files = { ...pack.files };
  for (const { path, language } of pack.components) if (options.trusted || language === "js") files[path] = "";
  return { version: pack.version, integrity: pack.integrity ?? null, files };
}
