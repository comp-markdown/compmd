import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { BlogError, configOf, readConfig } from "./config.ts";
import { slugOf } from "./slug.ts";

describe("configOf", () => {
  it("takes a title, an address and an author, with English when no language is given", () => {
    expect(configOf({ title: "Notes", url: "https://example.com/blog/", author: "Ada" })).toEqual({ title: "Notes", url: "https://example.com/blog/", author: "Ada", lang: "en" });
    const comments = { repo: "a/b", repoId: "R_1", category: "Blog", categoryId: "C_1" };
    expect(configOf({ title: "N", url: "https://example.com/", author: "A", lang: "de", description: "D", comments })).toMatchObject({ lang: "de", description: "D", comments });
  });

  it("names every field that is missing or wrong", () => {
    expect(() => configOf({ url: "https://example.com/blog" })).toThrow(new BlogError("blog.config.json: `title` is missing; `author` is missing; `url` is an http(s) address ending in `/`"));
    expect(() => configOf({ title: "N", url: "https://example.com/", author: "A", lang: 1, comments: { repo: "a/b" } })).toThrow(/`lang` is text; `comments` has `repo`, `repoId`/);
    expect(() => configOf(null)).toThrow(/`title` is missing/);
  });
});

describe("readConfig", () => {
  it("reads blog.config.json, and says what's wrong with a folder without one or a file that isn't JSON", async () => {
    const dir = await mkdtemp(join(tmpdir(), "compmd-blog-"));
    try {
      await expect(readConfig(dir)).rejects.toThrow(/has no blog\.config\.json: it is what makes a folder a blog/);
      await writeFile(join(dir, "blog.config.json"), "{ title");
      await expect(readConfig(dir)).rejects.toThrow(/blog\.config\.json isn't JSON/);
      await writeFile(join(dir, "blog.config.json"), JSON.stringify({ title: "N", url: "https://example.com/", author: "A" }));
      expect((await readConfig(dir)).title).toBe("N");
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});

describe("slugOf", () => {
  it("makes lowercase words joined by -, without accents or punctuation", () => {
    expect(slugOf("Why CompMD?")).toBe("why-compmd");
    expect(slugOf("Café & Crème, 2026")).toBe("cafe-creme-2026");
    expect(slugOf("  --  ")).toBe("");
  });
});
