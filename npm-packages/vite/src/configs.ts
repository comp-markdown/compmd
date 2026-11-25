// Which configuration a page gets (docs/vite-per-page-config.md, decisions 1 and 9): every folder under Vite's root
// that holds one, and the nearest of them above a module. Node only.

import { readdirSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { holdsConfig as holdsProjectConfig } from "@compmd/host/project";
import { nodeFiles } from "./node-files.ts";

/** Whether `dir` holds a configuration: `compmd.config.json`, or an object under `package.json`'s `compmd` key. */
export const holdsConfig = (dir: string): boolean => holdsProjectConfig(dir, nodeFiles);

/** A folder's path under the root, `/`-separated: `""` for the root, `playground` for a folder in it. */
export type ConfigFolder = string;

/**
 * The folders under `root` that hold a configuration, the root first (it always counts, with or without one), then
 * the others in path order. `skipped` are absolute folders not to look in, such as an `out` bundle or the build's
 * output; `node_modules` and hidden folders are never looked in.
 */
export function configFolders(root: string, skipped: readonly string[] = []): ConfigFolder[] {
  const found: ConfigFolder[] = [""];
  const skip = new Set(skipped.map((dir) => resolve(dir)));
  const walk = (dir: string): void => {
    for (const entry of readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      if (!entry.isDirectory() || entry.name === "node_modules" || entry.name.startsWith(".")) continue;
      const path = join(dir, entry.name);
      if (skip.has(path)) continue;
      if (holdsConfig(path)) found.push(relative(root, path).split(sep).join("/"));
      walk(path);
    }
  };
  walk(resolve(root));
  return found;
}

/**
 * The configuration folder of the module `file`: the nearest of `folders` at or above its own folder, up to the
 * root (`""`). A module outside the root, or none (a request without an importer), gets the root's.
 */
export function configFolderOf(file: string | undefined, root: string, folders: readonly ConfigFolder[]): ConfigFolder {
  if (file === undefined) return "";
  const path = relative(resolve(root), dirname(file.replace(/[?#].*$/, ""))).split(sep).join("/");
  if (path.startsWith("..")) return "";
  for (let folder = path; folder !== ""; folder = folder.includes("/") ? folder.slice(0, folder.lastIndexOf("/")) : "") {
    if (folders.includes(folder)) return folder;
  }
  return "";
}
