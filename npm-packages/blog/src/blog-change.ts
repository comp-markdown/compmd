// What a file saved in a blog changes, for a dev server that follows its saves (docs/blog-tool.md, decision 3).

import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { isAbsolute, join, relative, resolve, sep } from "node:path";
import { CONFIG } from "./config.ts";
import { THEME } from "./build.ts";
import { LAYOUT } from "./layout.ts";
import { PACKS } from "./live.ts";
import { POSTS } from "./posts.ts";

/**
 * What a saved file changes: `page`, the blog's pages, built again for each page asked for; `client`, the page's
 * script too, which a `ClientCache` keeps.
 */
export type BlogChange = "page" | "client";

/** Where a blog is made from: its folder, and the folder packs its `compmd.config.json` names, each absolute. */
export interface BlogSources {
  dir: string;
  folderPacks: string[];
}

/** The files of a blog's folder its page's script is made from, besides its folder packs. */
const CLIENT_FILES = new Set([PACKS, "package.json", "pnpm-lock.yaml", "package-lock.json"]);

/** The files of a blog's folder its pages are made from, besides its posts. */
const PAGE_FILES = new Set([CONFIG, THEME, LAYOUT]);

/** Where the blog in `dir` is made from, as its `compmd.config.json` says now. */
export async function blogSources(dir: string): Promise<BlogSources> {
  const config = join(dir, PACKS);
  const packs = existsSync(config) ? ((JSON.parse(await readFile(config, "utf8")) as { packs?: Record<string, unknown> }).packs ?? {}) : {};
  const folderPacks = Object.keys(packs)
    .filter((name) => name.startsWith("./") || name.startsWith("../"))
    .map((name) => resolve(dir, name));
  return { dir: resolve(dir), folderPacks };
}

/** `file`'s path in `folder`, `/`-separated, or `undefined` when it is outside it. */
function pathIn(folder: string, file: string): string | undefined {
  const path = relative(folder, file);
  return path === "" || path.startsWith("..") || isAbsolute(path) ? undefined : path.split(sep).join("/");
}

/** Whether a file is an editor's own, not the writer's: hidden (a swap file) or a backup (`post.comp.md~`). */
const isEditorFile = (path: string): boolean => path.split("/").some((part) => part.startsWith(".") || part.endsWith("~"));

/** What saving `file` (an absolute path) changes in the blog `sources` describes; `undefined` when it changes nothing. */
export function changeOf(sources: BlogSources, file: string): BlogChange | undefined {
  if (sources.folderPacks.some((pack) => pathIn(pack, file) !== undefined)) return "client";
  const path = pathIn(sources.dir, file);
  if (path === undefined || isEditorFile(path)) return undefined;
  if (CLIENT_FILES.has(path)) return "client";
  return PAGE_FILES.has(path) || path.startsWith(`${POSTS}/`) ? "page" : undefined;
}
