import { mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { blogSources, type BlogChange } from "./blog-change.ts";
import { watchBlog, type BlogWatch } from "./blog-watch.ts";
import { BLOG, writeBlog } from "./test-blog.ts";

let dir: string;
let watched: BlogWatch | undefined;
let changes: BlogChange[];

/** How far apart, in milliseconds, saves may come and still be one change, in these tests. */
const TOGETHER = 400;

beforeEach(async () => {
  dir = await writeBlog({ ...BLOG, "compmd.config.json": JSON.stringify({ packs: { "./components": {} } }), "components/index.comp.md": "" });
  await mkdir(join(dir, "node_modules"));
  changes = [];
  // Saves the test makes one after the other come together by the watcher's promise, not by the machine's speed: the
  // file events of two watchers can arrive further apart than `cblo dev`'s 50 ms on a loaded machine.
  watched = watchBlog(await blogSources(dir), (change) => changes.push(change), TOGETHER);
  // The system's file events take a moment to start, and may still bring the blog's own writing, just before.
  await new Promise((done) => setTimeout(done, 500));
  changes = [];
});

afterEach(async () => {
  watched?.close();
  await rm(dir, { recursive: true, force: true });
});

describe("watchBlog", () => {
  it("says once that the pages changed, for the saves of a post that come together", async () => {
    await writeFile(join(dir, "posts/first.comp.md"), "one");
    await writeFile(join(dir, "posts/first.comp.md"), "two");
    await vi.waitFor(() => { expect(changes).toEqual(["page"]); });
  });

  it("says the script changed when the packs' configuration or a folder pack does, with a post saved together", async () => {
    await writeFile(join(dir, "posts/second.comp.md"), "text");
    await writeFile(join(dir, "components/index.comp.md"), "changed");
    await vi.waitFor(() => { expect(changes).toEqual(["client"]); });
  });

  it("says nothing for a file no page is made from", async () => {
    await writeFile(join(dir, "node_modules/x.js"), "");
    await writeFile(join(dir, "README.md"), "");
    await writeFile(join(dir, "posts/first.comp.md"), "last");
    await vi.waitFor(() => { expect(changes).toEqual(["page"]); });
  });

  it("says nothing once it is closed", async () => {
    watched?.close();
    await writeFile(join(dir, "theme.css"), "body {}");
    await new Promise((done) => setTimeout(done, 200));
    expect(changes).toEqual([]);
  });
});
