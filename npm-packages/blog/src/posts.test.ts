import { rm } from "node:fs/promises";
import { describe, expect, it } from "vitest";
import { configOf } from "./config.ts";
import { postOf, readPosts } from "./posts.ts";
import { post, writeBlog } from "./test-blog.ts";

const CONFIG = configOf({ title: "N", url: "https://example.com/", author: "Ada" });

describe("postOf", () => {
  it("reads a post's frontmatter, its slug from its file, the blog's author when it names none", () => {
    const files = { "why.comp.md": post({ title: "Why", date: "2026-11-20", summary: "S", tags: ["a"], updated: "2026-11-21" }) };
    expect(postOf("why.comp.md", files, CONFIG)).toEqual({ slug: "why", path: "why.comp.md", title: "Why", date: "2026-11-20", updated: "2026-11-21", summary: "S", author: "Ada", tags: ["a"], draft: false, files });
  });

  it("names what a frontmatter lacks or has wrong", () => {
    const files = { "x.comp.md": post({ date: "20 Nov", updated: "soon", tags: "a", draft: "yes" }) };
    expect(() => postOf("x.comp.md", files, CONFIG)).toThrow("posts/x.comp.md: `title` is missing; `date` is a day, `YYYY-MM-DD`; `updated` is a day, `YYYY-MM-DD`; `tags` is a list of names; `draft` is `true` or `false`");
  });
});

describe("readPosts", () => {
  it("gives the posts newest first, drafts only when asked, and refuses a blog without posts", async () => {
    const dir = await writeBlog();
    try {
      expect((await readPosts(dir, CONFIG, { drafts: false })).map(({ slug }) => slug)).toEqual(["second", "first"]);
      expect((await readPosts(dir, CONFIG, { drafts: true })).map(({ slug }) => slug)).toEqual(["draft", "second", "first"]);
      const [second, first] = await readPosts(dir, CONFIG, { drafts: false });
      expect(Object.keys(second!.files)).toEqual(["second.comp.md"]);
      expect(Object.keys(first!.files).sort()).toEqual(["first.comp.md", "first/photo.svg"]);
      await expect(readPosts(`${dir}/none`, CONFIG, { drafts: false })).rejects.toThrow(/has no posts\/ folder/);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("gives a post the files a site shares with it, as its own", async () => {
    const dir = await writeBlog();
    try {
      const sharedFiles = { "first/shelf.js": "export default function mount() {}", "elsewhere.csv": "a\n1\n" };
      const [second, first] = await readPosts(dir, CONFIG, { drafts: false, sharedFiles });
      expect(first!.files["first/shelf.js"]).toBe(sharedFiles["first/shelf.js"]);
      // A shared file outside a post's folder is no post's.
      expect(Object.keys(second!.files)).toEqual(["second.comp.md"]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
