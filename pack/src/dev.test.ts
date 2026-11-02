// The test page's parts that aren't the browser's: its configuration and its HTML. The page itself is tested where
// `@compmd/create-pack` writes packs and opens them, in Chrome and WebKit.
import { rm } from "node:fs/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { pageConfig, pageHtml } from "./dev.ts";
import { readPack } from "./pack.ts";

let dir: string;

beforeEach(async () => {
  dir = await folder();
  await write(dir, { "package.json": { name: "@kit/charts", version: "0.1.0", compmd: "./index.comp.md" }, "index.comp.md": documentOf([]) });
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("the test page", () => {
  it("has the pack as a folder pack, with what the pack's own configuration adds", async () => {
    expect(await pageConfig(await readPack(dir))).toEqual({ packs: { "./": {} } });
    await write(dir, { "compmd.config.json": { libraries: ["@kit/three"], packs: { "@kit/three": {} } } });
    expect(await pageConfig(await readPack(dir))).toEqual({ libraries: ["@kit/three"], packs: { "./": {}, "@kit/three": {} } });
  });

  it("is titled by the pack, escaped, and follows the reader's scheme until switched", () => {
    const html = pageHtml("<b>@kit/charts</b> & co");
    expect(html).toContain("<title>&lt;b>@kit/charts&lt;/b> &amp; co</title>");
    expect(html).toContain(":root { color-scheme: light dark; }");
  });
});
