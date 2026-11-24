// A blog's posts (docs/blog-tool.md, decision 2): each CompMD document in `posts/`, its slug its file's name, what the
// pages need from its frontmatter, and the files beside it that it reads.

import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { frontmatterOf } from "@compmd/processor";
import { BlogError, type BlogConfig } from "./config.ts";
import { pathsUnder } from "./folder-paths.ts";
import { loadProcessor } from "./processor.ts";
import { slugOf } from "./slug.ts";
import { socialImageOf, type SocialImage } from "./social-image.ts";

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
  /** The image a link to it shows, `socialImage`, in place of the blog's default, when it has one. */
  socialImage?: SocialImage;
  /** Its files, by their `/`-separated paths in the posts' folder: its document, and those of its own folder (`<slug>/`). */
  files: Record<string, string>;
}

/** Whether `text` is a day of the calendar, `YYYY-MM-DD`: not `2026-13-45` (audit B17). */
const isDay = (text: string): boolean => /^\d{4}-\d{2}-\d{2}$/.test(text) && !Number.isNaN(Date.parse(`${text}T00:00:00Z`)) && new Date(`${text}T00:00:00Z`).toISOString().startsWith(text);

const UTF8 = new TextDecoder("utf-8", { fatal: true });

/**
 * The text files under `dir`, by their `/`-separated path there, leaving out dot files and files that aren't UTF-8
 * text (a photo): a post reads text, and an image loads by its URL (audit B18).
 */
async function filesUnder(dir: string): Promise<Record<string, string>> {
  const files: Record<string, string> = {};
  for (const path of await pathsUnder(dir)) {
    try {
      files[path] = UTF8.decode(await readFile(join(dir, ...path.split("/"))));
    } catch (error) {
      if (!(error instanceof TypeError)) throw error;
    }
  }
  return files;
}

/** What is wrong with a post's file name, as the slug in its address; `undefined` when it is a slug. */
function slugProblem(slug: string): string | undefined {
  if (slug !== "" && slugOf(slug) === slug) return undefined;
  const suggestion = slugOf(slug) === "" ? "" : `, such as \`${slugOf(slug)}${EXTENSION}\``;
  return `its file's name is its address: lowercase letters and digits, joined by \`-\`${suggestion}`;
}

/**
 * The post at `path` among `files`, its frontmatter read by the processor, the blog's author when it names none.
 *
 * @throws BlogError naming what its frontmatter lacks or has wrong.
 */
export function postOf(path: string, files: Record<string, string>, config: BlogConfig): Post {
  loadProcessor();
  const { doc } = frontmatterOf({ path, files, now: new Date(0).toISOString() });
  const slug = path.slice(0, -EXTENSION.length);
  const problems: string[] = [];
  const named = slugProblem(slug);
  if (named !== undefined) problems.push(named);
  const text = (name: string): string | undefined => (typeof doc[name] === "string" && doc[name].trim() !== "" ? doc[name] : undefined);
  const title = text("title");
  const date = text("date");
  if (title === undefined) problems.push("`title` is missing");
  if (date === undefined || !isDay(date)) problems.push("`date` is a day of the calendar, `YYYY-MM-DD`");
  const updated = text("updated");
  if (doc.updated !== undefined && (updated === undefined || !isDay(updated))) problems.push("`updated` is a day of the calendar, `YYYY-MM-DD`");
  const tags = doc.tags ?? [];
  if (!Array.isArray(tags) || !tags.every((tag) => typeof tag === "string" && tag.trim() !== "")) problems.push("`tags` is a list of names");
  else for (const tag of tags as string[]) if (slugOf(tag) === "") problems.push(`the tag ${JSON.stringify(tag)} has no letter or digit for its page's address`);
  if (doc.draft !== undefined && typeof doc.draft !== "boolean") problems.push("`draft` is `true` or `false`");
  const image = doc.socialImage === undefined ? undefined : socialImageOf(doc.socialImage, "socialImage", config.url);
  if (image && "problem" in image) problems.push(image.problem);
  if (problems.length > 0) throw new BlogError(`${POSTS}/${path}: ${problems.join("; ")}`);
  const post: Post = {
    slug,
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
  if (image && "image" in image) post.socialImage = image.image;
  return post;
}

/**
 * The posts of the blog in `dir`, newest first (by date, then by slug), with its drafts when `drafts`, and
 * `sharedFiles` among the posts' folder's files, by their path there.
 *
 * @throws BlogError when it has no posts' folder, or a post's frontmatter is wrong.
 */
export async function readPosts(dir: string, config: BlogConfig, { drafts, sharedFiles = {} }: { drafts: boolean; sharedFiles?: Record<string, string> }): Promise<Post[]> {
  let files: Record<string, string>;
  try {
    files = { ...(await filesUnder(join(dir, POSTS))), ...sharedFiles };
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
