// The Author's Guide's blog (chapter 19), built as `cblo build` builds it: its post's page, the list, the tag's page
// and the feed, with no warning.
import { join } from "node:path";
import { buildSite } from "@compmd/blog";
import { expect, it } from "vitest";

const BLOG = join(import.meta.dirname, "../authors/examples/blog");

it("builds the chapter's blog, its post with what it says", async () => {
  const { files, warnings } = await buildSite(BLOG, { drafts: false, now: "2026-11-21T09:00:00Z" });
  expect(warnings).toEqual([]);
  expect([...files.keys()]).toEqual(expect.arrayContaining(["index.html", "the-first-frost/index.html", "tags/winter/index.html", "feed.xml"]));
  const post = files.get("the-first-frost/index.html") as string;
  expect(post).toContain("<h1>The first frost</h1>");
  expect(post).toContain("Parsnips");
  expect(files.get("feed.xml")).toContain("https://example.org/garden/the-first-frost/");
});
