// Reading a project as `compmd-project` reads it (docs/language-server.md, Q3), for `@compmd/vite` and other builds:
// the configuration, the packs found, the lock file's integrity. In Node the module loads when first used; in a page
// or a worker, load it first, with `init()` from `@compmd/host`.

import {
  configOf as configOfJson,
  folderPacks as folderPacksJson,
  holdsConfig as holdsConfigOf,
  initSync,
  installedPacks as installedPacksJson,
  integrities as integritiesJson,
  lockFileOf as lockFileOfPath,
  readConfig as readConfigJson,
  rootOf as rootOfPath,
} from "../pkg/compmd_host_wasm.js";

/** Node's `process`, as far as it gives its built-in modules; absent in a page or a worker. */
type NodeProcess = { getBuiltinModule?: (id: "node:fs") => typeof import("node:fs") };

let loaded = false;

/**
 * Loads the module in Node, from its file, the first time a function is used (`getBuiltinModule` reaches Node's `fs`
 * without importing it, so this file stays a page's too). Elsewhere it is the host's to load, with `init()`.
 */
function ready(): void {
  if (loaded) return;
  loaded = true;
  const fs = (globalThis as { process?: NodeProcess }).process?.getBuiltinModule?.("node:fs");
  if (fs) initSync({ module: fs.readFileSync(new URL("../pkg/compmd_host_wasm_bg.wasm", import.meta.url)) });
}

/** A host's file system, as the reader asks it: a file's text, or `undefined`; whether a path is a folder. */
export interface ProjectFiles {
  text(path: string): string | undefined;
  isDir(path: string): boolean;
}

/** A pack's trust, as the processor takes it. */
export type ProjectTrust = { level: "strict" } | { level: "permissive"; network: string[]; host: boolean } | { level: "trusted" };

/** A configuration, read and checked, every default filled in; its packs in the order written. */
export interface ProjectConfig {
  packs: [string, { trust: ProjectTrust; ambient: boolean; version?: string }][];
  imports: "any" | "listed";
  deny: { pack: string; version?: string; reason: string }[];
  maxPackSize?: number;
  libraries: { name: string; imports: string[] }[];
  registries: Record<string, string>;
  fetch: "host" | "direct";
  origins: string[] | "any";
  editor: { suggest: string[] };
}

/** What reading a configuration gave: it, or every problem it has, or why the project couldn't be read. */
export type ConfigRead = { config: ProjectConfig } | { problems: string[]; source: string; message: string } | { error: string };

/** A pack found in a project. */
export interface FoundPack {
  name: string;
  version: string;
  /** Where its files are. */
  dir: string;
  /** The configuration's key, for a folder pack. */
  key?: string;
  /** Its integrity from the lock file, for an installed pack. */
  integrity?: string;
}

/** Packs found, or why a project's packs couldn't be read. */
export type PacksFound = { packs: FoundPack[] } | { error: string };

/** `value`, a parsed configuration, read and checked as coming from `source`. */
export function readConfig(value: unknown, source: string): ConfigRead {
  ready();
  return JSON.parse(readConfigJson(JSON.stringify(value === undefined ? null : value), source)) as ConfigRead;
}

/** The configuration of `folder`: `compmd.config.json`, or `package.json`'s `compmd` object, or the default. */
export function configOf(folder: string, files: ProjectFiles): ConfigRead {
  ready();
  return JSON.parse(configOfJson(folder, files)) as ConfigRead;
}

/** Whether `folder` holds a configuration. */
export function holdsConfig(folder: string, files: ProjectFiles): boolean {
  ready();
  return holdsConfigOf(folder, files);
}

/** The installed packs of the project at `root`: its scoped dependencies in `node_modules` that are packs. */
export function installedPacks(root: string, files: ProjectFiles): PacksFound {
  ready();
  return JSON.parse(installedPacksJson(root, files)) as PacksFound;
}

/** The folder packs among a configuration's pack `keys` (`./components`), relative to its `folder`. */
export function folderPacks(keys: string[], folder: string, files: ProjectFiles): PacksFound {
  ready();
  return JSON.parse(folderPacksJson(keys, folder, files)) as PacksFound;
}

/** The integrity of each release the lock file nearest above `root` holds, by `name@version`. */
export function integrities(root: string, files: ProjectFiles): Map<string, string> {
  ready();
  return new Map(Object.entries(JSON.parse(integritiesJson(root, files)) as Record<string, string>));
}

/** The lock file nearest above `root`, `pnpm-lock.yaml` or `package-lock.json`. */
export function lockFileOf(root: string, files: ProjectFiles): string | undefined {
  ready();
  return lockFileOfPath(root, files);
}

/** The root folder of the document at `document`: the nearest with a configuration, else the deepest of `workspaceFolders` that holds it, else its own. */
export function rootOf(document: string, workspaceFolders: string[], files: ProjectFiles): string {
  ready();
  return rootOfPath(document, workspaceFolders, files);
}
