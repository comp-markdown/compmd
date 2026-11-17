// What a blog gives besides its pages (docs/blog-tool.md, decision 4): an Atom feed of its latest posts, its posts as
// data (`posts.json`), and its sitemap.

import type { BlogConfig } from "./config.ts";
import { escape } from "./html.ts";
import { addressOf, type Page } from "./pages.ts";
import type { Post } from "./posts.ts";

/** How many of the latest posts the feed holds. */
export const FEED_SIZE = 20;

/** A day as RFC 3339 has it, at midnight UTC. */
const instant = (day: string): string => `${day}T00:00:00Z`;

/** The Atom feed of the latest posts, each with its HTML (`html` by slug) as its content. */
export function atom(config: BlogConfig, posts: readonly Post[], html: ReadonlyMap<string, string>): string {
  const latest = posts.slice(0, FEED_SIZE);
  const updated = latest.map((post) => post.updated ?? post.date).sort().at(-1) ?? "1970-01-01";
  const entries = latest.map((post) => {
    const address = addressOf(config, `${post.slug}/`);
    const categories = post.tags.map((tag) => `<category term="${escape(tag)}"/>`).join("");
    return [
      "<entry>",
      `<title>${escape(post.title)}</title>`,
      `<id>${escape(address)}</id>`,
      `<link rel="alternate" type="text/html" href="${escape(address)}"/>`,
      `<published>${instant(post.date)}</published>`,
      `<updated>${instant(post.updated ?? post.date)}</updated>`,
      `<author><name>${escape(post.author)}</name></author>`,
      categories,
      post.summary === "" ? "" : `<summary>${escape(post.summary)}</summary>`,
      `<content type="html">${escape(html.get(post.slug) ?? "")}</content>`,
      "</entry>",
    ].join("");
  });
  return [
    '<?xml version="1.0" encoding="utf-8"?>',
    `<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="${escape(config.lang)}">`,
    `<title>${escape(config.title)}</title>`,
    config.description === undefined ? "" : `<subtitle>${escape(config.description)}</subtitle>`,
    `<id>${escape(config.url)}</id>`,
    `<link rel="alternate" type="text/html" href="${escape(config.url)}"/>`,
    `<link rel="self" type="application/atom+xml" href="${escape(addressOf(config, "feed.xml"))}"/>`,
    `<updated>${instant(updated)}</updated>`,
    `<author><name>${escape(config.author)}</name></author>`,
    ...entries,
    "</feed>",
    "",
  ].join("\n");
}

/** A post as `posts.json` holds it. */
export interface PostData {
  title: string;
  date: string;
  updated?: string;
  summary: string;
  author: string;
  tags: string[];
  url: string;
}

/** The posts as data, newest first, for a site to show the blog's latest posts elsewhere. */
export function postsJson(config: BlogConfig, posts: readonly Post[]): string {
  const data = posts.map((post): PostData => {
    const entry: PostData = { title: post.title, date: post.date, summary: post.summary, author: post.author, tags: post.tags, url: addressOf(config, `${post.slug}/`) };
    if (post.updated !== undefined) entry.updated = post.updated;
    return entry;
  });
  return `${JSON.stringify(data, null, 2)}\n`;
}

/** The sitemap of `pages`, a post's with its last change. */
export function sitemap(config: BlogConfig, pages: readonly Page[]): string {
  const urls = pages.map(({ path, article }) => {
    const modified = article ? `<lastmod>${article.updated ?? article.date}</lastmod>` : "";
    return `<url><loc>${escape(addressOf(config, path))}</loc>${modified}</url>`;
  });
  return ['<?xml version="1.0" encoding="utf-8"?>', '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">', ...urls, "</urlset>", ""].join("\n");
}
