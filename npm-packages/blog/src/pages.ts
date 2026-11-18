// The pages a blog has (docs/blog-tool.md, decision 4): the list, a page per post, a page per tag and the tags'
// page, each its path in the blog, its title and description, and its content for the layout.

import type { BlogConfig } from "./config.ts";
import { escape } from "./html.ts";
import type { Post } from "./posts.ts";
import type { Rendered } from "./render.ts";
import { slugOf } from "./slug.ts";

/** A page of the blog, before the layout. */
export interface Page {
  /** Its folder in the blog, ending in `/`; `""` for the list. */
  path: string;
  title: string;
  description: string;
  /** What goes in the layout's `<!-- content -->`. */
  content: string;
  /** Its own stylesheet, a post's. */
  css?: string;
  /** For a post's page, what its Open Graph type and dates say. */
  article?: Post;
}

/** A tag's folder in the blog. */
export const tagPath = (tag: string): string => `tags/${slugOf(tag)}/`;

/** The blog's address for `path`, a folder in it. */
export const addressOf = (config: BlogConfig, path: string): string => new URL(path, config.url).href;

/** `path`, a folder in the blog, as the site's absolute path. */
const href = (config: BlogConfig, path: string): string => escape(new URL(path, config.url).pathname);

/** A day as readers read it, in the blog's language: `20 November 2026`. */
const dayOf = (config: BlogConfig, day: string): string =>
  new Intl.DateTimeFormat(config.lang, { day: "numeric", month: "long", year: "numeric", timeZone: "UTC" }).format(new Date(`${day}T00:00:00Z`));

/** A post's tags, each a link to its page: phrasing content, since they are in a paragraph. */
const tagsOf = (config: BlogConfig, post: Post): string =>
  post.tags.length === 0 ? "" : ` · <span class="tags">${post.tags.map((tag) => `<a href="${href(config, tagPath(tag))}">${escape(tag)}</a>`).join(" ")}</span>`;

/** A post's date, author and tags, as the list and its page show them. */
function metaOf(config: BlogConfig, post: Post): string {
  const updated = post.updated === undefined ? "" : `, updated <time datetime="${post.updated}">${escape(dayOf(config, post.updated))}</time>`;
  return `<p class="post-meta"><time datetime="${post.date}">${escape(dayOf(config, post.date))}</time>${updated} · ${escape(post.author)}${tagsOf(config, post)}</p>`;
}

/** A list of posts, each its title linked, its meta and its summary. */
function listOf(config: BlogConfig, posts: readonly Post[]): string {
  const items = posts.map((post) => {
    const summary = post.summary === "" ? "" : `<p>${escape(post.summary)}</p>`;
    return `<li><h2><a href="${href(config, `${post.slug}/`)}">${escape(post.title)}</a></h2>${metaOf(config, post)}${summary}</li>`;
  });
  return `<ul class="post-list">${items.join("")}</ul>`;
}

/** The list of every post, newest first. */
export function listPage(config: BlogConfig, posts: readonly Post[]): Page {
  return { path: "", title: config.title, description: config.description ?? config.title, content: `<h1>${escape(config.title)}</h1>${listOf(config, posts)}` };
}

/** Where a post's comments go, when the blog has them: giscus's script loads when a reader scrolls near them. */
function commentsOf(config: BlogConfig): string {
  const giscus = config.comments?.giscus;
  if (!giscus) return "";
  const attributes: Record<string, string> = {
    "data-repo": giscus.repo,
    "data-repo-id": giscus.repoId,
    "data-category": giscus.category,
    "data-category-id": giscus.categoryId,
    "data-mapping": "pathname",
    "data-strict": "1",
    "data-reactions-enabled": "1",
    "data-input-position": "top",
    "data-theme": "preferred_color_scheme",
    "data-lang": config.lang,
  };
  const set = Object.entries(attributes).map(([name, value]) => `s.setAttribute(${JSON.stringify(name)},${JSON.stringify(value)});`).join("");
  const load = `const c=document.getElementById("comments");new IntersectionObserver((e,o)=>{if(!e.some((x)=>x.isIntersecting))return;o.disconnect();const s=document.createElement("script");s.src="https://giscus.app/client.js";s.async=true;s.crossOrigin="anonymous";${set}c.append(s);},{rootMargin:"600px"}).observe(c);`;
  return `<section id="comments" class="giscus" aria-label="Comments"></section><script>${load}</script>`;
}

/** A post's page: its title, meta, the post, its neighbours, and its comments. */
export function postPage(config: BlogConfig, post: Post, rendered: Rendered, newer: Post | undefined, older: Post | undefined): Page {
  const link = (other: Post | undefined, rel: string, label: string) => (other ? `<a rel="${rel}" href="${href(config, `${other.slug}/`)}">${label} ${escape(other.title)}</a>` : "<span></span>");
  const nav = newer || older ? `<nav class="post-nav" aria-label="Posts">${link(older, "prev", "←")}${link(newer, "next", "→")}</nav>` : "";
  return {
    path: `${post.slug}/`,
    title: `${post.title} · ${config.title}`,
    description: post.summary || post.title,
    content: `<article class="post"><h1>${escape(post.title)}</h1>${metaOf(config, post)}<div class="post-body">${rendered.html}</div></article>${nav}${commentsOf(config)}`,
    css: rendered.css,
    article: post,
  };
}

/** Each tag the posts have, by name, with its posts, newest first; the tags in alphabetical order. */
export function tagsIn(posts: readonly Post[]): Map<string, Post[]> {
  const tags = new Map<string, Post[]>();
  for (const post of posts) for (const tag of post.tags) tags.set(tag, [...(tags.get(tag) ?? []), post]);
  return new Map([...tags].sort(([a], [b]) => a.localeCompare(b)));
}

/** A tag's page: its posts. */
export function tagPage(config: BlogConfig, tag: string, posts: readonly Post[]): Page {
  return { path: tagPath(tag), title: `${tag} · ${config.title}`, description: `Posts tagged ${tag}`, content: `<h1>Posts tagged ${escape(tag)}</h1>${listOf(config, posts)}` };
}

/** The tags' page: every tag, with how many posts it has. */
export function tagsPage(config: BlogConfig, tags: ReadonlyMap<string, readonly Post[]>): Page {
  const items = [...tags].map(([tag, posts]) => `<li><a href="${href(config, tagPath(tag))}">${escape(tag)}</a> (${String(posts.length)})</li>`);
  return { path: "tags/", title: `Tags · ${config.title}`, description: `The tags of ${config.title}`, content: `<h1>Tags</h1><ul class="tag-list">${items.join("")}</ul>` };
}
