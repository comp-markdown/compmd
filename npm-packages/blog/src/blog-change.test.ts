import { rm } from "node:fs/promises";
import { join, resolve } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { blogSources, changeOf, type BlogSources } from "./blog-change.ts";
import { BLOG, writeBlog } from "./test-blog.ts";

const DIR = resolve("/blogs/notes");
const SOURCES: BlogSources = { dir: DIR, folderPacks: [resolve("/blogs/app"), join(DIR, "components")] };
const at = (path: string) => join(DIR, path);

describe("changeOf", () => {
  it("says a post, a post's file, the configuration, the theme and the layout change the pages", () => {
    for (const path of ["posts/first.comp.md", "posts/first/photo.svg", "blog.config.json", "theme.css", "layout.html"]) expect(changeOf(SOURCES, at(path)), path).toBe("page");
  });

  it("says the packs' configuration, the blog's package and lock files and its folder packs change the script", () => {
    for (const path of ["compmd.config.json", "package.json", "pnpm-lock.yaml", "package-lock.json", "components/stars.svelte"]) expect(changeOf(SOURCES, at(path)), path).toBe("client");
    expect(changeOf(SOURCES, resolve("/blogs/app/index.comp.md"))).toBe("client");
  });

  it("leaves out what no page is made from: other files, an editor's own and what is outside the blog", () => {
    for (const path of ["dist/index.html", "README.md", "node_modules/x/index.js", "posts/.first.comp.md.swp", "posts/first.comp.md~"]) expect(changeOf(SOURCES, at(path)), path).toBeUndefined();
    expect(changeOf(SOURCES, resolve("/blogs/other/posts/first.comp.md"))).toBeUndefined();
    expect(changeOf(SOURCES, DIR)).toBeUndefined();
  });
});

describe("blogSources", () => {
  let dir: string;

  afterEach(async () => {
    await rm(dir, { recursive: true, force: true });
  });

  it("finds the folder packs the blog's compmd.config.json names, and none without one", async () => {
    dir = await writeBlog({ ...BLOG, "compmd.config.json": JSON.stringify({ packs: { "./components": {}, "../app": {}, "@kit/mermaid": {} } }) });
    expect(await blogSources(dir)).toEqual({ dir: resolve(dir), folderPacks: [join(dir, "components"), resolve(dir, "../app")] });
    await rm(join(dir, "compmd.config.json"));
    expect((await blogSources(dir)).folderPacks).toEqual([]);
  });
});
