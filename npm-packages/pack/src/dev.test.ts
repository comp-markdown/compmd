// The test page's parts that aren't the browser's: its configuration and its HTML. The page itself is tested where
// `@compmd/create-pack` writes packs and opens them, in Chrome and WebKit.
import { rm } from "node:fs/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { pageConfig, pageHtml, servePack } from "./dev.ts";
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

  it("gives the demo the pack's folder as its root folder: its data and documents, not its packages or its build", async () => {
    await write(dir, {
      "demo.comp.md": "# Demo\n",
      "walks.yaml": "- title: A walk\n",
      "notes/more.comp.md": "# More\n",
      "node_modules/x/README.md": "# Not the demo's\n",
      "dist/built.json": "{}",
    });
    const server = await servePack(await readPack(dir), { port: 4196 });
    try {
      const page = await server.transformRequest("/@compmd-pack/page.ts");
      const imported = [...(page?.code ?? "").matchAll(/from\s+"([^"?]+)\?[^"]*raw"/g)].map((match) => match[1]);
      expect(imported.sort()).toEqual(["/demo.comp.md", "/index.comp.md", "/notes/more.comp.md", "/package.json", "/walks.yaml"]);
    } finally {
      await server.close();
    }
  });

  it("is titled by the pack, escaped, and follows the reader's scheme until switched", () => {
    const html = pageHtml("<b>@kit/charts</b> & co");
    expect(html).toContain("<title>&lt;b>@kit/charts&lt;/b> &amp; co</title>");
    expect(html).toContain(":root { color-scheme: light dark; }");
  });
});
