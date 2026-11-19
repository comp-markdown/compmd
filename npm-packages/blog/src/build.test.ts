import { rm } from "node:fs/promises";
import { Window } from "happy-dom";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { buildSite, type Site } from "./build.ts";
import { BLOG, post, writeBlog } from "./test-blog.ts";

let dir: string;
let site: Site;
const NOW = "2026-11-25T12:00:00Z";

/** A built file's text. */
const text = (path: string): string => {
  const content = site.files.get(path);
  if (typeof content !== "string") throw new Error(`no ${path}`);
  return content;
};

/** A built page as a document. */
const page = (path: string): Document => {
  const window = new Window();
  window.document.write(text(path));
  return window.document as unknown as Document;
};

beforeAll(async () => {
  dir = await writeBlog();
  site = await buildSite(dir, { drafts: false, now: NOW });
});

afterAll(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("buildSite", () => {
  it("writes the list, a page per post, the tags, the feed, the posts as data, the sitemap and the posts' files", () => {
    expect([...site.files.keys()].sort()).toEqual([
      "feed.xml",
      "first/index.html",
      "first/photo.svg",
      "index.html",
      "posts.json",
      "second/index.html",
      "sitemap.xml",
      "style.css",
      "tags/code-notes/index.html",
      "tags/index.html",
      "tags/intro/index.html",
    ]);
    expect(site.warnings).toEqual([]);
  });

  it("lists the posts newest first, each with its date, author, tags and summary, escaped", () => {
    const list = page("index.html");
    expect(list.title).toBe("Notes & Code");
    expect([...list.querySelectorAll(".post-list h2 a")].map((link) => [link.textContent, link.getAttribute("href")])).toEqual([
      ["Second <post>", "/blog/second/"],
      ["First", "/blog/first/"],
    ]);
    expect(list.querySelector(".post-list .post-meta")?.textContent).toBe("November 10, 2026, updated November 12, 2026 · Grace · intro Code Notes");
    expect([...list.querySelectorAll(".post-list .tags a")].map((link) => link.getAttribute("href"))).toEqual(["/blog/tags/intro/", "/blog/tags/code-notes/", "/blog/tags/intro/"]);
    expect(list.querySelector('link[rel="alternate"]')?.getAttribute("href")).toBe("/blog/feed.xml");
    expect(list.querySelector('link[rel="stylesheet"]')?.getAttribute("href")).toBe("/blog/style.css");
  });

  it("renders a post with CompMD, its code highlighted, its relative links one folder up, between its neighbours", () => {
    const second = page("second/index.html");
    expect(second.querySelector("article h1")?.textContent).toBe("Second <post>");
    expect(second.querySelector(".post-body .compmd-box, .post-body [class*='compmd-b']")?.textContent).toContain("In a box.");
    expect(second.querySelector("pre.shiki code")?.textContent).toBe("const a = 1;");
    expect(second.querySelector("style")?.textContent).toContain("compmd");
    expect(second.querySelector('a[rel="prev"]')?.getAttribute("href")).toBe("/blog/first/");
    expect(second.querySelector('meta[property="og:type"]')?.getAttribute("content")).toBe("article");
    const first = page("first/index.html");
    expect(first.querySelector(".post-body img")?.getAttribute("src")).toBe("../first/photo.svg");
    expect(first.querySelector('.post-body a[href="https://example.com/"]')).not.toBeNull();
    expect(first.querySelector('a[rel="next"]')?.getAttribute("href")).toBe("/blog/second/");
    expect(first.querySelector("#comments")).toBeNull();
  });

  it("writes each tag's page and the tags' page", () => {
    expect([...page("tags/index.html").querySelectorAll(".tag-list li")].map((item) => item.textContent)).toEqual(["Code Notes (1)", "intro (2)"]);
    expect([...page("tags/intro/index.html").querySelectorAll(".post-list h2")].map((heading) => heading.textContent)).toEqual(["Second <post>", "First"]);
  });

  it("writes the feed with each post's HTML, the posts as data and the sitemap", () => {
    const feed = text("feed.xml");
    expect(feed).toContain('<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="en">');
    expect(feed).toContain("<title>Second &lt;post&gt;</title>");
    expect(feed).toContain("<id>https://example.com/blog/second/</id>");
    expect(feed).toContain('<category term="Code Notes"/>');
    expect(feed).toContain("<updated>2026-11-12T00:00:00Z</updated>");
    expect(feed).toContain("&lt;pre class=&quot;shiki");
    expect(JSON.parse(text("posts.json"))).toEqual([
      { title: "Second <post>", date: "2026-11-10", updated: "2026-11-12", summary: "Code and a box.", author: "Grace", tags: ["intro", "Code Notes"], url: "https://example.com/blog/second/" },
      { title: "First", date: "2026-11-01", summary: "The first post.", author: "Ada", tags: ["intro"], url: "https://example.com/blog/first/" },
    ]);
    expect(text("sitemap.xml")).toContain("<url><loc>https://example.com/blog/second/</loc><lastmod>2026-11-12</lastmod></url>");
  });

  it("builds drafts when asked, a blog's own layout and theme, and comments", async () => {
    const config = { title: "N", url: "https://example.com/", author: "A", comments: { giscus: { repo: "a/b", repoId: "R", category: "Blog", categoryId: "C" } } };
    const own = await writeBlog({ ...BLOG, "blog.config.json": JSON.stringify(config), "layout.html": "<html><head></head><body><nav>mine</nav><!-- content --></body></html>", "theme.css": ".mine {}" });
    try {
      const built = await buildSite(own, { drafts: true, now: NOW });
      expect(built.files.has("draft/index.html")).toBe(true);
      const draft = built.files.get("draft/index.html") as string;
      expect(draft).toContain("<nav>mine</nav>");
      expect(draft).toContain('<section id="comments"');
      expect(draft).toContain('"data-mapping","pathname"');
      expect(built.files.get("style.css")).toContain(".mine {}");
      // A layout given, as a site gives the one it makes, in place of the blog's own.
      const inSiteLayout = await buildSite(own, { drafts: false, now: NOW, layout: "<html><head></head><body><nav>the site's</nav><!-- content --></body></html>" });
      expect(inSiteLayout.files.get("index.html")).toContain("<nav>the site's</nav>");
      expect(inSiteLayout.files.get("index.html")).not.toContain("<nav>mine</nav>");
    } finally {
      await rm(own, { recursive: true, force: true });
    }
  });

  it("fails on a post with an error, naming it, and on a layout without its content", async () => {
    const broken = await writeBlog({ ...BLOG, "posts/broken.comp.md": post({ title: "Broken", date: "2026-11-02" }, ":: nowhere[Nothing]\n") });
    const noContent = await writeBlog({ ...BLOG, "layout.html": "<html><head></head><body></body></html>" });
    try {
      await expect(buildSite(broken, { drafts: false, now: NOW })).rejects.toThrow(/posts\/broken\.comp\.md has errors:\n- posts\/broken\.comp\.md:\d+:\d+: F002 unknown component `nowhere`/);
      await expect(buildSite(noContent, { drafts: false, now: NOW })).rejects.toThrow("layout.html has no `<!-- content -->` for each page's content");
    } finally {
      await rm(broken, { recursive: true, force: true });
      await rm(noContent, { recursive: true, force: true });
    }
  });
});
