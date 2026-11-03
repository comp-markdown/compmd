import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { guidePages } from "./guide-pages.ts";
import { sectionsOf, sitePages } from "./search.ts";

describe("sectionsOf", () => {
  it("makes a section of each heading, with its id in its address, and of the text before the first", () => {
    const html =
      '<p>Before.</p><h2 id="data">Data <a class="anchor" href="#data">#</a><a class="source" href="x">Source</a></h2><p>From <code>files</code>.</p><h3 id="csv">CSV</h3><ul><li>Rows.</li></ul>';
    expect(sectionsOf([{ url: "spec/", page: "The spec", html }])).toEqual([
      { id: 0, url: "spec/", page: "The spec", title: "The spec", text: "Before." },
      { id: 1, url: "spec/#data", page: "The spec", title: "Data", text: "From files ." },
      { id: 2, url: "spec/#csv", page: "The spec", title: "CSV", text: "Rows." },
    ]);
  });

  it("leaves out the site's header, footer and navigation, and an empty start", () => {
    const html = '<header class="site"><nav>Spec Guides</nav></header><h1>CompMD</h1><p>Markdown with components.</p><footer>Copyright</footer>';
    expect(sectionsOf([{ url: "", page: "CompMD", html }])).toEqual([{ id: 0, url: "", page: "CompMD", title: "CompMD", text: "Markdown with components." }]);
  });
});

// Rendered once, as `guide-pages.test.ts` does: every chapter, which takes longer than a test's time while all the
// packages' tests run.
const GUIDES = join(import.meta.dirname, "../../guides");
const pages = await guidePages((path) => readFileSync(join(GUIDES, path), "utf8"), "https://github.com/comp-markdown/compmd/blob/main/");

describe("sitePages", () => {
  it("names each guide page by its book and chapter", () => {
    const named = sitePages("<p>Landing.</p>", "<h2 id=\"a\">A</h2>", pages);
    expect(named.slice(0, 2).map(({ url, page }) => ({ url, page }))).toEqual([
      { url: "", page: "CompMD" },
      { url: "spec/", page: "The spec" },
    ]);
    expect(named.find(({ url }) => url === "guides/")?.page).toBe("The guides");
    expect(named.find(({ url }) => url === "guides/authors/")?.page).toBe("CompMD Author's Guide");
    expect(named.find(({ url }) => url === "guides/developers/quick-start/")?.page).toBe("CompMD Developer's Guide: 2. Quick start");
  });
});
