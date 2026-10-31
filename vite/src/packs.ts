// Finding the app's packs and reading each as the processor takes it (docs/vite.md, "What the plugin does"): every
// installed dependency that is a pack, and the folder packs the configuration lists; each pack's documents and the files
// they import, its integrity from the app's lock file, and its size.

import { existsSync } from "node:fs";
import { readFile, realpath, stat } from "node:fs/promises";
import { dirname, join, posix, resolve } from "node:path";
import type { PackRelease } from "@compmd/processor";
import { parseAllDocuments, parse as parseYaml } from "yaml";

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
  /** Every file its documents need, by path in the pack, with its text. */
  files: Record<string, string>;
  /** The component files its documents import: built modules (`.js`) and source. */
  components: ComponentFile[];
  /** The integrity the registry published for it, from the app's lock file. */
  integrity?: string;
  /** The size of its files, in bytes. */
  size: number;
}

/** What went wrong finding a pack: a folder that isn't one, a file a document imports that isn't there. */
export class PackError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "PackError";
  }
}

const LANGUAGES: Record<string, ComponentFile["language"]> = { ".jsx": "jsx", ".tsx": "tsx", ".svelte": "svelte", ".js": "js", ".ts": "ts" };
const FRONTMATTER = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/;

/** The language of a component file by its extension, or `undefined` for a file that isn't one. */
const languageOf = (path: string): ComponentFile["language"] | undefined => LANGUAGES[posix.extname(path)];

/** The relative paths a document's frontmatter names: its component and document imports, its data files. */
function importsOf(text: string): string[] {
  const frontmatter = FRONTMATTER.exec(text)?.[1];
  if (frontmatter === undefined) return [];
  let md: unknown;
  try {
    md = (parseYaml(frontmatter) as { md?: unknown } | null)?.md;
  } catch {
    // The processor reports a broken frontmatter, from the text it is given.
    return [];
  }
  if (typeof md !== "object" || md === null) return [];
  const { import: imports, data } = md as { import?: unknown; data?: unknown };
  const entries: unknown[] = [imports, data].flatMap((list): unknown[] => (Array.isArray(list) ? (list as unknown[]) : []));
  const froms = entries.map((entry) => (typeof entry === "object" && entry !== null ? (entry as { from?: unknown }).from : undefined));
  return froms.filter((from): from is string => typeof from === "string" && (from.startsWith("./") || from.startsWith("../")));
}

/** The `package.json` at `dir`, or `undefined` when there is none. */
async function manifestAt(dir: string): Promise<Record<string, unknown> | undefined> {
  try {
    return JSON.parse(await readFile(join(dir, "package.json"), "utf8")) as Record<string, unknown>;
  } catch {
    return undefined;
  }
}

/** A pack's document path, from its manifest (spec §6.4). */
const documentOf = (manifest: Record<string, unknown>): string => (typeof manifest.compmd === "string" ? posix.normalize(manifest.compmd) : "index.comp.md");

/**
 * Reads the pack at `dir`: its manifest, its document, and every file its documents import, recursively.
 *
 * @throws PackError when `dir` has no `package.json` with a name and a version, or a document imports a file that isn't
 *   in the pack.
 */
export async function readPack(dir: string): Promise<Pack> {
  const manifest = await manifestAt(dir);
  if (!manifest || typeof manifest.name !== "string") throw new PackError(`${dir} isn't a pack: it has no package.json with a name`);
  const name = manifest.name;
  const version = typeof manifest.version === "string" ? manifest.version : "0.0.0";
  const document = documentOf(manifest);
  const files: Record<string, string> = { "package.json": await readFile(join(dir, "package.json"), "utf8") };
  const components: ComponentFile[] = [];
  let size = Buffer.byteLength(files["package.json"] ?? "");
  const queue = [document];
  for (let path = queue.shift(); path !== undefined; path = queue.shift()) {
    if (path in files) continue;
    if (path.startsWith("../")) throw new PackError(`${name}: \`${path}\` leaves the pack`);
    let text: string;
    try {
      text = await readFile(join(dir, ...path.split("/")), "utf8");
    } catch {
      throw new PackError(`${name}: \`${path}\` isn't in the pack${path === document ? ", which needs its document (package.json's `compmd`, or index.comp.md)" : ""}`);
    }
    size += Buffer.byteLength(text);
    const language = languageOf(path);
    if (language) components.push({ path, language });
    files[path] = text;
    if (path.endsWith(".comp.md")) queue.push(...importsOf(text).map((from) => posix.normalize(posix.join(posix.dirname(path), from))));
  }
  return { name, version, dir, document, files, components, size };
}

/** Whether the package at `dir` is a pack: its manifest names a document, or it has `index.comp.md`. */
async function isPack(dir: string): Promise<boolean> {
  const manifest = await manifestAt(dir);
  return manifest !== undefined && (typeof manifest.compmd === "string" || existsSync(join(dir, "index.comp.md")));
}

/** The app's direct dependencies, by name. */
async function dependenciesOf(root: string): Promise<string[]> {
  const manifest = (await manifestAt(root)) ?? {};
  const names = (field: unknown) => (typeof field === "object" && field !== null ? Object.keys(field) : []);
  return [...new Set([...names(manifest.dependencies), ...names(manifest.devDependencies)])].sort();
}

/**
 * Finds the app's packs: every scoped dependency in `node_modules` that is a pack (docs/vite.md, decision 3), and the
 * folder packs `folders` names, relative to `root`.
 *
 * @throws PackError when a folder isn't a pack.
 */
export async function findPacks(root: string, folders: readonly string[]): Promise<Pack[]> {
  const packs: Pack[] = [];
  for (const name of await dependenciesOf(root)) {
    const dir = join(root, "node_modules", ...name.split("/"));
    if (name.startsWith("@") && existsSync(dir) && (await isPack(dir))) packs.push(await readPack(await realpath(dir)));
  }
  for (const folder of folders) {
    const dir = resolve(root, folder);
    if (!existsSync(dir) || !(await stat(dir)).isDirectory()) throw new PackError(`${folder} is listed in the configuration, but it isn't a folder of the app`);
    packs.push({ ...(await readPack(dir)), folder });
  }
  return packs;
}

/** The lock file nearest above `root`: a workspace keeps one for all its packages. */
function lockFileOf(root: string): string | undefined {
  for (let dir = resolve(root); ; dir = dirname(dir)) {
    for (const name of ["pnpm-lock.yaml", "package-lock.json"]) if (existsSync(join(dir, name))) return join(dir, name);
    if (dirname(dir) === dir) return undefined;
  }
}

/**
 * The integrity the registry published for each package the app's lock file holds, by `name@version`: from
 * `pnpm-lock.yaml` or `package-lock.json`, the nearest above `root` (docs/vite.md, V1). Empty without a lock file.
 */
export async function integrities(root: string): Promise<Map<string, string>> {
  const file = lockFileOf(root);
  const found = new Map<string, string>();
  if (!file) return found;
  const text = await readFile(file, "utf8");
  if (file.endsWith(".yaml")) {
    // pnpm writes its settings as a document of their own before the lock itself.
    type Lock = { packages?: Record<string, { resolution?: { integrity?: string } }> } | null;
    const lock = parseAllDocuments(text)
      .map((document) => document.toJS() as Lock)
      .find((document) => document?.packages !== undefined);
    for (const [key, entry] of Object.entries(lock?.packages ?? {})) {
      // `@kit/mermaid@1.0.0`, or `/@kit/mermaid@1.0.0` in older lock files; a peer suffix `(react@19.0.0)` follows.
      const id = key.replace(/^\//, "").replace(/\(.*$/, "");
      const integrity = entry.resolution?.integrity;
      if (integrity) found.set(id, integrity);
    }
  } else {
    const lock = JSON.parse(text) as { packages?: Record<string, { version?: string; integrity?: string; name?: string }> };
    for (const [path, entry] of Object.entries(lock.packages ?? {})) {
      const name = entry.name ?? path.split("node_modules/").at(-1);
      if (name && entry.version && entry.integrity) found.set(`${name}@${entry.version}`, entry.integrity);
    }
  }
  return found;
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
