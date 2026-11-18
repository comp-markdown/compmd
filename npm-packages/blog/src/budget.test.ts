// The pages' size budget (docs/blog-tool.md, decision 8, the CompMD website's landing page's): each page with its
// stylesheet at most 100 KB gzipped before its live parts, so it paints quickly over a slow connection.

import { rm } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { buildSite, type Site } from "./build.ts";
import { writeBlog } from "./test-blog.ts";

const BUDGET = 100 * 1024;

let dir: string;
let site: Site;

beforeAll(async () => {
  dir = await writeBlog();
  site = await buildSite(dir, { drafts: false, now: "2026-11-25T12:00:00Z" });
});

afterAll(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("the pages' size", () => {
  it("is within the budget for every page, with its stylesheet, before its live parts", () => {
    const style = site.files.get("style.css") ?? "";
    for (const [path, content] of site.files) {
      if (!path.endsWith("index.html")) continue;
      const size = gzipSync(Buffer.concat([Buffer.from(content), Buffer.from(style)])).length;
      expect(size, path).toBeLessThan(BUDGET);
    }
  });
});
