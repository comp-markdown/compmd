// A blog built (docs/blog-tool.md, decisions 3 to 6): its pages, its stylesheet, its feed, its posts as data, its
// sitemap and its posts' own files, each by its path in the site; `cblo build` writes them, `cblo dev` serves them.

import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { readConfig, type BlogConfig } from "./config.ts";
import { pathsUnder } from "./folder-paths.ts";
import { atom, postsJson, sitemap } from "./feeds.ts";
import { highlighter } from "./highlight.ts";
import { escape, inScript, inStyle } from "./html.ts";
import { applyLayout, defaultLayout, LAYOUT } from "./layout.ts";
import { addressOf, listPage, postPage, tagPage, tagsIn, tagsPage, type Page } from "./pages.ts";
import { POST_DATA, POST_FOLDER } from "./client-data.ts";
import type { ClientCache } from "./client-cache.ts";
import { buildClient, packsOf, type Client } from "./live.ts";
import { blogOrigins } from "./origins.ts";
import { EXTENSION, POSTS, readPosts, type Post } from "./posts.ts";
import { renderPost } from "./render.ts";
import { SiteFiles } from "./site-files.ts";
import { socialImageTags } from "./social-image.ts";

/** A blog's own stylesheet's file, after the default's. */
export const THEME = "theme.css";

/** What a build takes. */
export interface BuildOptions {
  /** Whether drafts are built too: `cblo dev`'s default. */
  drafts: boolean;
  /** The render time, RFC 3339: now when not given. */
  now?: string;
  /** Where the page's script is kept between builds of this blog, by a dev server; built for each build without it. */
  clients?: ClientCache;
  /**
   * The layout, in place of the blog's `layout.html`: for a site that makes it from its own pages' header and footer
   * as it builds them, so the blog's pages stay the same as the site's.
   */
  layout?: string;
  /**
   * Files a site shares with its posts from elsewhere in it, by their path in the posts' folder
   * (`hello-world/shelf.js`): a post reads them as its own, so the site keeps one copy.
   */
  sharedFiles?: Record<string, string>;
}

/** A blog built: each file by its path in the site, and the build's warnings. */
export interface Site {
  files: Map<string, string | Uint8Array>;
  warnings: string[];
}

/** The default stylesheet, shipped with the package. */
const DEFAULT_THEME = new URL("../theme/default.css", import.meta.url);

/** A page's head: its title, description, address, Open Graph tags and social image, the feed, and its stylesheets. */
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
    ...socialImageTags(page.article?.socialImage ?? config.defaultSocialImage),
    `<link rel="alternate" type="application/atom+xml" title="${escape(config.title)}" href="${escape(new URL("feed.xml", config.url).pathname)}">`,
    `<link rel="stylesheet" href="${escape(new URL("style.css", config.url).pathname)}">`,
    ...(page.css ? [`<style>${inStyle(page.css)}</style>`] : []),
  ];
  return lines.join("\n");
}

/**
 * The files of the posts' folder that aren't posts, by their `/`-separated path there, as bytes: never a dot file, nor
 * the folder of a post left out, `hidden` (a draft's, audit S29).
 */
async function postFiles(dir: string, hidden: readonly Post[]): Promise<Map<string, Uint8Array>> {
  const folder = join(dir, POSTS);
  const folders = hidden.map((post) => `${post.slug}/`);
  const files = new Map<string, Uint8Array>();
  for (const path of await pathsUnder(folder)) {
    if (path.endsWith(EXTENSION) || folders.some((left) => path.startsWith(left))) continue;
    files.set(path, await readFile(join(folder, ...path.split("/"))));
  }
  return files;
}

/**
 * What a live post's page adds: its document and files for the script, and the origins it reaches without asking a
 * reader, as the build rendered it with, as JSON; and the script.
 */
function liveOf(post: Post, script: string, origins: string[] | "any"): string {
  const data = inScript(JSON.stringify({ path: post.path, files: post.files, origins }));
  return `<script type="application/json" id="${POST_DATA}">${data}</script><script type="module" src="${escape(script)}"></script>`;
}

/** The blog in `dir`, built. @throws BlogError for what is wrong with it, a post's errors among them. */
export async function buildSite(dir: string, { drafts, now = new Date().toISOString(), clients, layout, sharedFiles = {} }: BuildOptions): Promise<Site> {
  const config = await readConfig(dir);
  const every = await readPosts(dir, config, { drafts: true, sharedFiles });
  const posts = every.filter((post) => drafts || !post.draft);
  const pageLayout = layout ?? (existsSync(join(dir, LAYOUT)) ? await readFile(join(dir, LAYOUT), "utf8") : defaultLayout(config));
  const theme = [await readFile(DEFAULT_THEME, "utf8"), existsSync(join(dir, THEME)) ? await readFile(join(dir, THEME), "utf8") : ""].join("\n");
  const highlight = await highlighter();
  const blogPacks = await packsOf(dir);
  const origins = blogOrigins(config, blogPacks.allowedOrigins);
  const given = { ...blogPacks, allowedOrigins: origins };
  const base = new URL(config.url).pathname;
  const warnings: string[] = [];
  const feedHtml = new Map<string, string>();
  const pages: { page: Page; owner: string }[] = [{ page: listPage(config, posts), owner: "the list of posts" }];
  const site = new SiteFiles();
  for (const [path, content] of await postFiles(dir, every.filter((post) => !posts.includes(post)))) site.set(path, content, `${POSTS}/${path}`);
  let client: Client | undefined;
  for (const [i, post] of posts.entries()) {
    // A post's page is a folder below the posts', so its relative links take one `../` more; the feed's are absolute.
    const rendered = await renderPost(post, now, highlight, given, { up: POST_FOLDER, base: config.url });
    warnings.push(...rendered.warnings);
    feedHtml.set(post.slug, rendered.feedHtml);
    const page = postPage(config, post, rendered, posts[i - 1], posts[i + 1]);
    if (rendered.live) {
      if (!client) {
        const building = () => buildClient(dir, base);
        client = await (clients ? clients.get(building) : building());
        for (const [path, content] of client.files) site.set(path, content, "the page's script");
      }
      page.content += liveOf(post, `${base}${client.script}`, origins);
    }
    pages.push({ page, owner: `the page of ${POSTS}/${post.path}` });
  }
  const tags = tagsIn(posts);
  if (tags.size > 0) pages.push({ page: tagsPage(config, tags), owner: "the tags' page" }, ...[...tags].map(([tag, tagged]) => ({ page: tagPage(config, tag, tagged), owner: `the page of the tag ${JSON.stringify(tag)}` })));
  for (const { page, owner } of pages) site.set(`${page.path}index.html`, applyLayout(pageLayout, { head: headOf(config, page), content: page.content }), owner);
  site.set("style.css", theme, "the stylesheet");
  site.set("feed.xml", atom(config, posts, feedHtml), "the feed");
  site.set("posts.json", postsJson(config, posts), "the posts as data");
  site.set("sitemap.xml", sitemap(config, pages.map(({ page }) => page)), "the sitemap");
  return { files: site.files, warnings };
}
