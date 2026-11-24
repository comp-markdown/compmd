// The files of a blog's folder, as the build reads them (audit S29): the posts and their files, never a dot file or
// what a dot folder holds (`.DS_Store`, `.git/`, an editor's swap file), which a blog never publishes.

import { readdir } from "node:fs/promises";
import { join, relative, sep } from "node:path";

/** The files under `dir`, by their `/`-separated path there, leaving out dot files and dot folders. */
export async function pathsUnder(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => relative(dir, join(entry.parentPath, entry.name)).split(sep).join("/"))
    .filter((path) => !path.split("/").some((part) => part.startsWith(".")));
}
