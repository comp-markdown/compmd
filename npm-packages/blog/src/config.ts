// A blog's `blog.config.json` (docs/blog-tool.md, decision 2): its title, address, author and language, and its
// comments.

import { readFile } from "node:fs/promises";
import { join } from "node:path";

/** A blog's comments, by their provider: giscus alone for now, each other a key of its own (decision 7). */
export interface Comments {
  giscus: Giscus;
}

/** Comments as a repository's GitHub Discussions, through giscus: what giscus.app gives for the repository. */
export interface Giscus {
  /** The repository, `owner/name`. */
  repo: string;
  /** Its id, as giscus.app gives it. */
  repoId: string;
  /** The Discussions category the posts' discussions are in. */
  category: string;
  /** Its id, as giscus.app gives it. */
  categoryId: string;
}

/** A blog's configuration. */
export interface BlogConfig {
  title: string;
  /** Its address, ending in `/`: `https://host/blog/`. */
  url: string;
  /** Who writes it, when a post doesn't say. */
  author: string;
  /** Its language, a BCP 47 tag: `en` when it doesn't say. */
  lang: string;
  /** One sentence on what it is, for the list's description and the feed's subtitle. */
  description?: string;
  /** Its comments; none when it has no `comments`. */
  comments?: Comments;
}

/** What is wrong with a blog: its configuration, a post, a command's arguments. Its message says what to fix. */
export class BlogError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BlogError";
  }
}

/** The configuration's file in a blog's folder. */
export const CONFIG = "blog.config.json";

const isText = (value: unknown): value is string => typeof value === "string" && value.trim() !== "";

/** `given`, read as a blog's configuration. @throws BlogError naming every field that is missing or wrong. */
export function configOf(given: unknown): BlogConfig {
  const fields = (typeof given === "object" && given !== null ? given : {}) as Record<string, unknown>;
  const problems: string[] = [];
  for (const name of ["title", "url", "author"] as const) if (!isText(fields[name])) problems.push(`\`${name}\` is missing`);
  if (isText(fields.url) && !/^https?:\/\/[^/]+\/(.*\/)?$/.test(fields.url)) problems.push("`url` is an http(s) address ending in `/`");
  for (const name of ["lang", "description"] as const) if (fields[name] !== undefined && !isText(fields[name])) problems.push(`\`${name}\` is text`);
  const comments = fields.comments as Record<string, unknown> | undefined;
  const giscus = comments?.giscus as Record<string, unknown> | undefined;
  if (comments !== undefined && Object.keys(comments).some((name) => name !== "giscus")) problems.push("`comments` names its provider: `giscus` is the one there is");
  else if (comments !== undefined && !["repo", "repoId", "category", "categoryId"].every((name) => isText(giscus?.[name]))) {
    problems.push("`comments.giscus` has `repo`, `repoId`, `category` and `categoryId`, as giscus.app gives them");
  }
  if (problems.length > 0) throw new BlogError(`${CONFIG}: ${problems.join("; ")}`);
  const config: BlogConfig = { title: fields.title as string, url: fields.url as string, author: fields.author as string, lang: (fields.lang as string | undefined) ?? "en" };
  if (isText(fields.description)) config.description = fields.description;
  if (giscus !== undefined) config.comments = { giscus: { repo: giscus.repo as string, repoId: giscus.repoId as string, category: giscus.category as string, categoryId: giscus.categoryId as string } };
  return config;
}

/** The configuration of the blog in `dir`. @throws BlogError when it is missing, isn't JSON, or is wrong. */
export async function readConfig(dir: string): Promise<BlogConfig> {
  let text: string;
  try {
    text = await readFile(join(dir, CONFIG), "utf8");
  } catch {
    throw new BlogError(`${dir} has no ${CONFIG}: it is what makes a folder a blog`);
  }
  try {
    return configOf(JSON.parse(text));
  } catch (error) {
    if (error instanceof BlogError) throw error;
    throw new BlogError(`${CONFIG} isn't JSON: ${error instanceof Error ? error.message : String(error)}`);
  }
}
