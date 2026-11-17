// A blog built (docs/blog-tool.md, decisions 3 to 6): its pages, its stylesheet, its feed, its posts as data, its
// sitemap and its posts' own files, each by its path in the site; `cblo build` writes them, `cblo dev` serves them.

import { existsSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join, relative, sep } from "node:path";
import { readConfig, type BlogConfig } from "./config.ts";
import { atom, postsJson, sitemap } from "./feeds.ts";
import { highlighter } from "./highlight.ts";
import { escape } from "./html.ts";
import { applyLayout, defaultLayout, LAYOUT } from "./layout.ts";
import { addressOf, listPage, postPage, tagPage, tagsIn, tagsPage, type Page } from "./pages.ts";
import { POST_DATA } from "./client-data.ts";
import { buildClient, packsOf, type Client } from "./live.ts";
import { EXTENSION, POSTS, readPosts, type Post } from "./posts.ts";
import { renderPost } from "./render.ts";

/** A blog's own stylesheet's file, after the default's. */
export const THEME = "theme.css";

/** What a build takes. */
export interface BuildOptions {
  /** Whether drafts are built too: `cblo dev`'s default. */
  drafts: boolean;
  /** The render time, RFC 3339: now when not given. */
  now?: string;
  /** Whether the page's script, once built, is kept for the next build of the same blog: `cblo dev`'s. */
  reuseClient?: boolean;
}

/** A blog built: each file by its path in the site, and the build's warnings. */
export interface Site {
  files: Map<string, string | Uint8Array>;
  warnings: string[];
}

/** The default stylesheet, shipped with the package. */
const DEFAULT_THEME = new URL("../theme/default.css", import.meta.url);

/** A page's head: its title, description, address, Open Graph tags, the feed, and its stylesheets. */
function headOf(config: BlogConfig, page: Page): string {
  const address = addressOf(config, page.path);
  const lines = [
    `<title>${escape(page.title)}</title>`,
    `<meta name="description" content="${escape(page.description)}">`,
    `<link rel="canonical" href="${escape(address)}">`,
    `<meta property="og:title" content="${escape(page.article?.title ?? page.title)}">`,
    `<meta property="og:description" content="${escape(page.description)}">`,
    `<meta property="og:url" content="${escape(address)}">`,
    `<meta property="og:type" content="${page.article ? "article" : "website"}">`,
    ...(page.article ? [`<meta property="article:published_time" content="${page.article.date}">`] : []),
    `<link rel="alternate" type="application/atom+xml" title="${escape(config.title)}" href="${escape(new URL("feed.xml", config.url).pathname)}">`,
    `<link rel="stylesheet" href="${escape(new URL("style.css", config.url).pathname)}">`,
    ...(page.css ? [`<style>${page.css}</style>`] : []),
  ];
  return lines.join("\n");
}

/** The files of the posts' folder that aren't posts, by their `/`-separated path there, as bytes. */
async function postFiles(dir: string): Promise<Map<string, Uint8Array>> {
  const folder = join(dir, POSTS);
  const entries = await readdir(folder, { recursive: true, withFileTypes: true });
  const files = new Map<string, Uint8Array>();
  for (const entry of entries.filter((found) => found.isFile() && !found.name.endsWith(EXTENSION))) {
    const path = join(entry.parentPath, entry.name);
    files.set(relative(folder, path).split(sep).join("/"), await readFile(path));
  }
  return files;
}

/** The page's script, built once for each blog while a `cblo dev` runs. */
const clients = new Map<string, Promise<Client>>();

/** What a live post's page adds: its document and files for the script, as JSON, and the script. */
function liveOf(post: Post, script: string): string {
  // `<` written as `<`, so nothing in the post's text ends the element.
  const data = JSON.stringify({ path: post.path, files: post.files }).replace(/</g, "\\u003c");
  return `<script type="application/json" id="${POST_DATA}">${data}</script><script type="module" src="${escape(script)}"></script>`;
}

/** The blog in `dir`, built. @throws BlogError for what is wrong with it, a post's errors among them. */
export async function buildSite(dir: string, { drafts, now = new Date().toISOString(), reuseClient = false }: BuildOptions): Promise<Site> {
  const config = await readConfig(dir);
  const posts = await readPosts(dir, config, { drafts });
  const layout = existsSync(join(dir, LAYOUT)) ? await readFile(join(dir, LAYOUT), "utf8") : defaultLayout(config);
  const theme = [await readFile(DEFAULT_THEME, "utf8"), existsSync(join(dir, THEME)) ? await readFile(join(dir, THEME), "utf8") : ""].join("\n");
  const highlight = await highlighter();
  const given = await packsOf(dir);
  const base = new URL(config.url).pathname;
  const warnings: string[] = [];
  const html = new Map<string, string>();
  const pages: Page[] = [listPage(config, posts)];
  const files = new Map<string, string | Uint8Array>(await postFiles(dir));
  let client: Client | undefined;
  for (const [i, post] of posts.entries()) {
    // A post's page is a folder below the posts', so its relative links take one `../` more.
    const rendered = await renderPost(post, now, highlight, given, "../");
    warnings.push(...rendered.warnings);
    html.set(post.slug, rendered.html);
    const page = postPage(config, post, rendered, posts[i - 1], posts[i + 1]);
    if (rendered.live) {
      if (!client) {
        const building = (reuseClient ? clients.get(dir) : undefined) ?? buildClient(dir, base);
        if (reuseClient) clients.set(dir, building);
        client = await building;
        for (const [path, content] of client.files) files.set(path, content);
      }
      page.content += liveOf(post, `${base}${client.script}`);
    }
    pages.push(page);
  }
  const tags = tagsIn(posts);
  if (tags.size > 0) pages.push(tagsPage(config, tags), ...[...tags].map(([tag, tagged]) => tagPage(config, tag, tagged)));
  for (const page of pages) files.set(`${page.path}index.html`, applyLayout(layout, { head: headOf(config, page), content: page.content }));
  files.set("style.css", theme);
  files.set("feed.xml", atom(config, posts, html));
  files.set("posts.json", postsJson(config, posts));
  files.set("sitemap.xml", sitemap(config, pages));
  return { files, warnings };
}
