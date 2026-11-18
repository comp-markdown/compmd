import { readFile, readdir, rm } from "node:fs/promises";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "./cli.ts";
import { fileFor, serve } from "./serve.ts";
import { writeBlog } from "./test-blog.ts";

let dir: string;

beforeEach(async () => {
  dir = await writeBlog();
  vi.spyOn(console, "log").mockImplementation(() => undefined);
  vi.spyOn(console, "error").mockImplementation(() => undefined);
});

afterEach(async () => {
  vi.restoreAllMocks();
  await rm(dir, { recursive: true, force: true });
});

describe("cblo new", () => {
  it("writes a draft with its frontmatter filled in, and refuses a post that is there or a title with no slug", async () => {
    expect(await run(["new", "Why CompMD?"], dir, "2026-11-25")).toBe(0);
    expect(await readFile(join(dir, "posts/why-compmd.comp.md"), "utf8")).toBe('---\ntitle: "Why CompMD?"\ndate: 2026-11-25\nauthor: "Ada"\nsummary: ""\ntags: []\ndraft: true\n---\n\n# Why CompMD?\n\n');
    expect(await run(["new", "Why CompMD?"], dir, "2026-11-25")).toBe(1);
    expect(console.error).toHaveBeenCalledWith(expect.stringMatching(/why-compmd\.comp\.md is there already/));
    expect(await run(["new", "Other", "--slug", "my-other"], dir, "2026-11-25")).toBe(0);
    expect((await readdir(join(dir, "posts"))).sort()).toContain("my-other.comp.md");
    expect(await run(["new", "?!"], dir, "2026-11-25")).toBe(1);
    expect(await run(["new", "X", "--slug", "Not A Slug"], dir, "2026-11-25")).toBe(1);
  });
});

describe("cblo build", () => {
  it("writes the blog into its folder, without drafts", async () => {
    expect(await run(["build", "--out", join(dir, "out")], dir, "2026-11-25")).toBe(0);
    expect((await readdir(join(dir, "out"))).sort()).toEqual(["feed.xml", "first", "index.html", "posts.json", "second", "sitemap.xml", "style.css", "tags"]);
  });

  it("says how it is used when the command is wrong", async () => {
    expect(await run([], dir, "2026-11-25")).toBe(2);
    expect(await run(["publish"], dir, "2026-11-25")).toBe(2);
    expect(console.error).toHaveBeenCalledWith(expect.stringContaining("cblo new <title>"));
  });
});

describe("cblo dev", () => {
  it("serves the blog at its path, drafts included unless --no-drafts, built again for each request", async () => {
    const server = await serve(dir, 0, { drafts: true });
    try {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      const get = (path: string) => fetch(`http://127.0.0.1:${String(port)}${path}`, { redirect: "manual" });
      expect((await get("/")).headers.get("location")).toBe("/blog/");
      const draft = await get("/blog/draft/");
      expect(draft.headers.get("content-type")).toBe("text/html; charset=utf-8");
      expect(await draft.text()).toContain("<h1>Draft</h1>");
      expect((await get("/blog/first/photo.svg")).headers.get("content-type")).toBe("image/svg+xml");
      expect((await get("/blog/nothing/")).status).toBe(404);
    } finally {
      server.close();
    }
  });

  it("finds a folder's index.html, and nothing outside the blog's path", () => {
    const site = { files: new Map<string, string>([["index.html", "list"], ["a b/index.html", "post"]]), warnings: [] };
    expect(fileFor(site, "/blog/", "/blog/")).toBe("index.html");
    expect(fileFor(site, "/blog/", "/blog/a%20b/")).toBe("a b/index.html");
    expect(fileFor(site, "/blog/", "/other/")).toBeUndefined();
  });
});
