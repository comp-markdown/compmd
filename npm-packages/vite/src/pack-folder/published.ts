// What a pack publishes, as npm packs it: what `package.json`'s `files` names (globs and folders), and what npm always
// adds. Only the pack's own files: a match outside its folder is dropped.

import { glob, readdir, stat } from "node:fs/promises";
import { join, relative, resolve, sep } from "node:path";
import { isPackPath } from "./confined.ts";

/** The files npm puts in every package, whatever its `files` says. */
const ALWAYS = /^(package\.json|readme|license|licence|changelog)(\..*)?$/i;

/** What in a pack's folder isn't its files: its own dependencies, and hidden files and folders (`.git`). */
export const isSkipped = (name: string): boolean => name === "node_modules" || name.startsWith(".");

/** Every file under the folder `path` of the pack at `dir`, by its path in the pack, but skipped ones. */
async function filesUnder(dir: string, path: string): Promise<string[]> {
  const files: string[] = [];
  for (const entry of await readdir(join(dir, ...path.split("/")), { withFileTypes: true })) {
    if (isSkipped(entry.name)) continue;
    const inside = `${path}/${entry.name}`;
    if (entry.isDirectory()) files.push(...(await filesUnder(dir, inside)));
    else if (entry.isFile()) files.push(inside);
  }
  return files;
}

/** `match`, a path `glob` gave from `dir`, as a path in the pack; `undefined` for one outside it. */
function inPack(dir: string, match: string): string | undefined {
  const path = relative(dir, resolve(dir, match)).split(sep).join("/");
  return isPackPath(path) ? path : undefined;
}

/**
 * The files the pack at `dir` publishes, by their paths in the pack, sorted: those its `files` entries match, a folder's
 * files under it, and what npm always adds; `undefined` when `files` is no list, which publishes the whole folder.
 * A `!` entry, which leaves files out in npm, is ignored: leaving none out only serves more of what the pack holds. A
 * match outside the pack, which a link or a pattern through `..` could give, is dropped.
 *
 * @example
 * await publishedFiles("compmd-packs/mermaid", ["dist/*.js", "index.comp.md"]);
 * // ["README.md", "dist/mermaid.js", "index.comp.md", "package.json"]
 */
export async function publishedFiles(dir: string, files: unknown): Promise<string[] | undefined> {
  if (!Array.isArray(files)) return undefined;
  const published = new Set<string>();
  for (const entry of await readdir(dir)) if (ALWAYS.test(entry)) published.add(entry);
  // A pattern through `..` names nothing of the pack, as in npm.
  const patterns = files.filter((entry): entry is string => typeof entry === "string" && !entry.startsWith("!") && !entry.split(/[\\/]/).includes(".."));
  for (const pattern of patterns) {
    for await (const match of glob(pattern.replace(/^\.?\/+/, "").replace(/\/+$/, ""), { cwd: dir })) {
      const path = inPack(dir, match);
      if (path === undefined || path.split("/").some(isSkipped)) continue;
      if ((await stat(join(dir, ...path.split("/")))).isDirectory()) for (const file of await filesUnder(dir, path)) published.add(file);
      else published.add(path);
    }
  }
  return [...published].sort();
}
