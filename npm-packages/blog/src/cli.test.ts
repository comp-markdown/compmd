import { readFile, readdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { run } from "./cli.ts";
import { RELOAD_PATH } from "./reload-events.ts";
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
    expect(await readFile(join(dir, "posts/why-compmd.comp.md"), "utf8")).toBe('---\ntitle: "Why CompMD?"\ndate: 2026-11-25\nauthor: "Ada"\nsummary: ""\ntags: []\ndraft: true\n---\n\n');
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

  it("gives its pages the script that reloads them, tells them when a post is saved, and shows a mistake on the page", async () => {
    const server = await serve(dir, 0, { drafts: true });
    const stream = new AbortController();
    try {
      const address = server.address();
      const origin = `http://127.0.0.1:${String(typeof address === "object" && address ? address.port : 0)}`;
      const page = await (await fetch(`${origin}/blog/first/`)).text();
      expect(page).toContain(`new EventSource("${RELOAD_PATH}")`);
      expect(await (await fetch(`${origin}/blog/first/photo.svg`)).text()).not.toContain("EventSource");
      const events = await fetch(`${origin}${RELOAD_PATH}`, { signal: stream.signal });
      expect(events.headers.get("content-type")).toBe("text/event-stream");
      const reader = (events.body as ReadableStream<Uint8Array>).getReader();
      const next = async () => new TextDecoder().decode((await reader.read()).value);
      expect(await next()).toBe(": open\n\n");
      await writeFile(join(dir, "posts/first.comp.md"), "---\ntitle: First\ndate: 2026-11-01\nsummary: Changed.\n---\n\n::: box\nNot closed.\n");
      expect(await next()).toBe("data: reload\n\n");
      const mistake = await fetch(`${origin}/blog/first/`);
      expect(mistake.status).toBe(500);
      expect(await mistake.text()).toMatch(/first\.comp\.md[\s\S]*EventSource/);
    } finally {
      stream.abort();
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
