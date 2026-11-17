// A blog's posts (docs/blog-tool.md, decision 2): each CompMD document in `posts/`, its slug its file's name, what the
// pages need from its frontmatter, and the files beside it that it reads.

import { readdir, readFile } from "node:fs/promises";
import { join, relative, sep } from "node:path";
import { frontmatterOf } from "@compmd/processor";
import { BlogError, type BlogConfig } from "./config.ts";
import { loadProcessor } from "./processor.ts";

/** A blog's posts' folder. */
export const POSTS = "posts";
/** A post's extension. */
export const EXTENSION = ".comp.md";

/** A post. */
export interface Post {
  /** Its address in the blog, its file's name without `.comp.md`. */
  slug: string;
  /** Its document's path among {@link files}: `<slug>.comp.md`. */
  path: string;
  title: string;
  /** The day it was published, `YYYY-MM-DD`. */
  date: string;
  /** The day it last changed after that, when it has one. */
  updated?: string;
  summary: string;
  author: string;
  tags: string[];
  draft: boolean;
  /** Its files, by their `/`-separated paths in the posts' folder: its document, and those of its own folder (`<slug>/`). */
  files: Record<string, string>;
}

const DAY = /^\d{4}-\d{2}-\d{2}$/;

/** The files under `dir`, by their `/`-separated path there. */
async function filesUnder(dir: string): Promise<Record<string, string>> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  const files: Record<string, string> = {};
  for (const entry of entries.filter((found) => found.isFile())) {
    const path = join(entry.parentPath, entry.name);
    files[relative(dir, path).split(sep).join("/")] = await readFile(path, "utf8");
  }
  return files;
}

/**
 * The post at `path` among `files`, its frontmatter read by the processor, the blog's author when it names none.
 *
 * @throws BlogError naming what its frontmatter lacks or has wrong.
 */
export function postOf(path: string, files: Record<string, string>, config: BlogConfig): Post {
  loadProcessor();
  const { doc } = frontmatterOf({ path, files, now: new Date(0).toISOString() });
  const problems: string[] = [];
  const text = (name: string): string | undefined => (typeof doc[name] === "string" && doc[name].trim() !== "" ? doc[name] : undefined);
  const title = text("title");
  const date = text("date");
  if (title === undefined) problems.push("`title` is missing");
  if (date === undefined || !DAY.test(date)) problems.push("`date` is a day, `YYYY-MM-DD`");
  const updated = text("updated");
  if (doc.updated !== undefined && (updated === undefined || !DAY.test(updated))) problems.push("`updated` is a day, `YYYY-MM-DD`");
  const tags = doc.tags ?? [];
  if (!Array.isArray(tags) || !tags.every((tag) => typeof tag === "string" && tag.trim() !== "")) problems.push("`tags` is a list of names");
  if (doc.draft !== undefined && typeof doc.draft !== "boolean") problems.push("`draft` is `true` or `false`");
  if (problems.length > 0) throw new BlogError(`${POSTS}/${path}: ${problems.join("; ")}`);
  const post: Post = {
    slug: path.slice(0, -EXTENSION.length),
    path,
    title: title ?? "",
    date: date ?? "",
    summary: text("summary") ?? "",
    author: text("author") ?? config.author,
    tags: tags as string[],
    draft: doc.draft === true,
    files,
  };
  if (updated !== undefined) post.updated = updated;
  return post;
}

/**
 * The posts of the blog in `dir`, newest first (by date, then by slug), with its drafts when `drafts`.
 *
 * @throws BlogError when it has no posts' folder, or a post's frontmatter is wrong.
 */
export async function readPosts(dir: string, config: BlogConfig, { drafts }: { drafts: boolean }): Promise<Post[]> {
  let files: Record<string, string>;
  try {
    files = await filesUnder(join(dir, POSTS));
  } catch {
    throw new BlogError(`${dir} has no ${POSTS}/ folder for its posts`);
  }
  /** A post's own files: its document, and its folder's. */
  const own = (path: string): Record<string, string> => {
    const folder = `${path.slice(0, -EXTENSION.length)}/`;
    return Object.fromEntries(Object.entries(files).filter(([file]) => file === path || file.startsWith(folder)));
  };
  const posts = Object.keys(files)
    .filter((path) => path.endsWith(EXTENSION) && !path.includes("/"))
    .map((path) => postOf(path, own(path), config));
  return posts.filter((post) => drafts || !post.draft).sort((a, b) => b.date.localeCompare(a.date) || a.slug.localeCompare(b.slug));
}
