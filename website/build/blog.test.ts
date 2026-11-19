// @vitest-environment node
// The site's blog: its layout made from a page template and the site's header and footer, and the blog built in it,
// with its posts for the search index and its pages' folders for the sitemap.
import { CONTENT_MARKER, readConfig, readPosts } from "@compmd/blog";
import { describe, expect, it } from "vitest";
import { BLOG_FOLDER, blogLayout, buildBlog } from "./blog.ts";

const TEMPLATE = `<!doctype html><html><head><link rel="icon" href="../logo/icon.svg"><link rel="stylesheet" href="../assets/spec.css"><!--head--></head><body><!--app--><script type="module" src="../assets/spec.js"></script></body></html>`;

describe("blogLayout", () => {
  const layout = blogLayout(TEMPLATE, ({ content }) => ({ head: "", body: `<header class="site">The site</header><main>${content}</main>` }), (href) => `/* ${href} */`);

  it("puts the site's header around the blog tool's content marker", () => {
    expect(layout).toContain(`<header class="site">The site</header><main>${CONTENT_MARKER}</main>`);
  });

  it("inlines the template's stylesheets, and starts its other links at the site's root, for pages at any depth", () => {
    expect(layout).toContain("<style>/* ../assets/spec.css */</style>");
    expect(layout).toContain('href="/logo/icon.svg"');
    expect(layout).toContain('src="/assets/spec.js"');
    expect(layout).not.toContain('="../');
  });
});

describe("buildBlog", () => {
  const LAYOUT = `<html><head></head><body><nav class="site">The site</nav>${CONTENT_MARKER}</body></html>`;

  it("builds every page in the layout, each post for the search index, and the pages' folders for the sitemap", async () => {
    const { site, posts, folders } = await buildBlog({ layout: LAYOUT, drafts: false });
    const published = await readPosts(BLOG_FOLDER, await readConfig(BLOG_FOLDER), { drafts: false });
    expect(published.length).toBeGreaterThan(0);
    expect(posts.map(({ url, page }) => [url, page])).toEqual(published.map((post) => [`blog/${post.slug}/`, `Blog: ${post.title}`]));
    for (const { html } of posts) expect(html).toContain('<nav class="site">The site</nav>');
    expect(folders).toContain("blog/");
    expect(folders).toEqual(expect.arrayContaining(published.map((post) => `blog/${post.slug}/`)));
    expect(site.files.has("feed.xml")).toBe(true);
  }, 30_000);
});
