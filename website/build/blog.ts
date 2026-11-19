// The site's blog (website/docs/blog.md): the folder `blog/`, built by `@compmd/blog` in a layout made from the site's
// own header and footer, so its pages are the site's. The prerender writes it into `dist/blog/`, with its posts in the
// search index and its pages in the sitemap; the dev server serves it from the working tree.
import { buildSite, CONTENT_MARKER, readConfig, readPosts, type BuildOptions, type Site } from "@compmd/blog";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { prerendered, relocated, type Rendered } from "./prerender.ts";
import type { IndexedPage } from "./search.ts";

/** The blog's folder, as `@compmd/blog` reads one. */
export const BLOG_FOLDER = join(import.meta.dirname, "../blog");

/** The blog's folder in the site. */
export const BLOG_PATH = "blog/";

/** The site's folder. */
const SITE_FOLDER = join(import.meta.dirname, "..");

/**
 * The files of the site a post uses as its own, by their path in the posts' folder, each from its place in the site:
 * the book club of the landing page's hero, so the site keeps one copy of it.
 */
const SHARED_FILES: Record<string, string> = {
  "hello-world/shelf.js": "content/shelf.js",
  "hello-world/club.csv": "content/club.csv",
};

/** The files the posts share with the site, read from it. */
const sharedFiles = async (): Promise<Record<string, string>> => {
  const readSharedFile = async ([path, file]: [string, string]): Promise<[string, string]> => [path, await readFile(join(SITE_FOLDER, file), "utf8")];
  return Object.fromEntries(await Promise.all(Object.entries(SHARED_FILES).map(readSharedFile)));
};

/**
 * The blog's layout: the page `template` (`blog/index.html`, as built) with the site's header and footer that
 * `renderLayout` renders around the blog tool's content marker, its stylesheets inlined by `readStylesheet` as the
 * prerender inlines them. Its other links then start at the site's root, since one layout serves the blog's pages at
 * every depth.
 */
export function blogLayout(template: string, renderLayout: (props: { content: string }) => Rendered, readStylesheet: (href: string) => string): string {
  return relocated(prerendered(template, renderLayout({ content: CONTENT_MARKER }), readStylesheet), "/");
}

/** The blog, built: its files by their path in the blog, its posts as the search index takes pages, and its pages' folders in the site. */
export interface BuiltBlog {
  site: Site;
  posts: IndexedPage[];
  folders: string[];
}

/** The blog built as `options` say, in their layout (`blogLayout`). @throws BlogError for what is wrong with it. */
export async function buildBlog(options: BuildOptions & { layout: string }): Promise<BuiltBlog> {
  const shared = await sharedFiles();
  const site = await buildSite(BLOG_FOLDER, { ...options, sharedFiles: shared });
  const posts = await readPosts(BLOG_FOLDER, await readConfig(BLOG_FOLDER), { drafts: options.drafts, sharedFiles: shared });
  const pageOf = (path: string): string => {
    const page = site.files.get(path);
    if (typeof page !== "string") throw new Error(`the blog has no page ${path}`);
    return page;
  };
  return {
    site,
    posts: posts.map((post) => ({ url: `${BLOG_PATH}${post.slug}/`, page: `Blog: ${post.title}`, html: pageOf(`${post.slug}/index.html`) })),
    folders: [...site.files.keys()].filter((path) => path === "index.html" || path.endsWith("/index.html")).map((path) => `${BLOG_PATH}${path.slice(0, -"index.html".length)}`),
  };
}
