import { rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
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

  it("reads a post's own social image, its address in the blog", () => {
    const files = { "why.comp.md": post({ title: "Why", date: "2026-11-20", socialImage: { url: "why/cover.png", alt: "A cover" } }) };
    expect(postOf("why.comp.md", files, CONFIG).socialImage).toEqual({ url: "https://example.com/why/cover.png", alt: "A cover" });
  });

  it("names what a frontmatter lacks or has wrong", () => {
    const files = { "x.comp.md": post({ date: "20 Nov", updated: "soon", tags: "a", draft: "yes", socialImage: { url: "c.png" } }) };
    expect(() => postOf("x.comp.md", files, CONFIG)).toThrow(
      "posts/x.comp.md: `title` is missing; `date` is a day of the calendar, `YYYY-MM-DD`; `updated` is a day of the calendar, `YYYY-MM-DD`; `tags` is a list of names; `draft` is `true` or `false`; `socialImage` has a `url` (an http(s) address, or one in the blog) and an `alt`",
    );
  });

  it("refuses a day that isn't one of the calendar, naming the post", () => {
    for (const date of ["2026-13-45", "2026-02-30"]) {
      const files = { "x.comp.md": post({ title: "X", date }) };
      expect(() => postOf("x.comp.md", files, CONFIG), date).toThrow("posts/x.comp.md: `date` is a day of the calendar, `YYYY-MM-DD`");
    }
    expect(postOf("x.comp.md", { "x.comp.md": post({ title: "X", date: "2028-02-29" }) }, CONFIG).date).toBe("2028-02-29");
  });

  it("refuses a file name that isn't a slug, and a tag with no letter or digit, as neither makes an address", () => {
    const named = { "My Post.comp.md": post({ title: "X", date: "2026-11-20" }) };
    expect(() => postOf("My Post.comp.md", named, CONFIG)).toThrow("posts/My Post.comp.md: its file's name is its address: lowercase letters and digits, joined by `-`, such as `my-post.comp.md`");
    const tagged = { "x.comp.md": post({ title: "X", date: "2026-11-20", tags: ["C", "+++"] }) };
    expect(() => postOf("x.comp.md", tagged, CONFIG)).toThrow('posts/x.comp.md: the tag "+++" has no letter or digit for its page\'s address');
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

  it("gives a post its folder's text files, and not a file that isn't text, such as a photo", async () => {
    const dir = await writeBlog();
    try {
      await writeFile(join(dir, "posts", "first", "photo.png"), new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0xff, 0xfe]));
      const [, first] = await readPosts(dir, CONFIG, { drafts: false });
      expect(Object.keys(first!.files).sort()).toEqual(["first.comp.md", "first/photo.svg"]);
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
