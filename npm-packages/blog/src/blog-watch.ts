// `cblo dev`'s watch of the blog's files (docs/blog-tool.md, decision 3).

import { existsSync, watch, type FSWatcher } from "node:fs";
import { join, sep } from "node:path";
import { changeOf, type BlogChange, type BlogSources } from "./blog-change.ts";
import { POSTS } from "./posts.ts";

/** A watch of a blog, until it is closed. */
export interface BlogWatch {
  close(): void;
}

/** How long a save takes to settle, in milliseconds: an editor writes a file in steps, each an event. */
const SETTLE = 50;

/** The more a change changes: a change of the script's sources is one of the pages' too. */
const larger = (one: BlogChange | undefined, other: BlogChange): BlogChange => (one === "client" ? one : other);

/**
 * Watches the blog `sources` describes: its own files, its posts and its folder packs, but not its `node_modules`.
 * `onChange` is called once for the saves that come together, with what they change: saves `settle` milliseconds or
 * less apart, the last as long ago (`SETTLE`, 50, by default, so `cblo dev` answers a save at once).
 */
export function watchBlog(sources: BlogSources, onChange: (change: BlogChange) => void, settle = SETTLE): BlogWatch {
  let pending: BlogChange | undefined;
  let timer: NodeJS.Timeout | undefined;
  const saw = (file: string) => {
    const change = changeOf(sources, file);
    if (change === undefined) return;
    pending = larger(pending, change);
    clearTimeout(timer);
    timer = setTimeout(() => {
      const settled = pending ?? change;
      pending = undefined;
      onChange(settled);
    }, settle);
  };
  const watching = (folder: string, recursive: boolean): FSWatcher[] =>
    existsSync(folder)
      ? [
          watch(folder, { recursive }, (_event, name) => {
            if (name !== null) saw(join(folder, name));
          }),
        ]
      : [];
  const posts = join(sources.dir, POSTS);
  const packs = sources.folderPacks.filter((pack) => !pack.startsWith(`${posts}${sep}`));
  const watchers = [...watching(sources.dir, false), ...watching(posts, true), ...packs.flatMap((pack) => watching(pack, true))];
  return {
    close() {
      clearTimeout(timer);
      for (const watcher of watchers) watcher.close();
    },
  };
}
