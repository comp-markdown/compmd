import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { descriptionOf, guidePages } from "./guide-pages.ts";

const GUIDES = join(import.meta.dirname, "../../guides");
const BLOB = "https://github.com/comp-markdown/compmd/blob/main/";
const pages = await guidePages((path) => readFileSync(join(GUIDES, path), "utf8"), BLOB);
const page = (folder: string) => pages.find((candidate) => candidate.folder === folder);
const htmlOf = (folder: string): string => String(page(folder)?.props.html);

describe("descriptionOf", () => {
  it("is the first paragraph of prose, as plain text", () => {
    expect(descriptionOf("# Quick start\n\n> [!NOTE]\n> Aside.\n\nRender a **document** in [a page](./page.md), with `render`.\n\nMore.\n")).toBe("Render a document in a page, with render.");
  });

  it("is cut at a word, about two hundred characters in", () => {
    const description = descriptionOf(`# Long\n\n${"word ".repeat(100)}\n`);
    expect(description.length).toBeLessThanOrEqual(201);
    expect(description).toMatch(/word…$/);
  });
});

describe("guidePages", () => {
  it("gives each page a description of its own", () => {
    const descriptions = pages.map(({ props }) => String(props.description));
    expect(descriptions.every((description) => description.length > 20)).toBe(true);
    expect(new Set(descriptions).size).toBe(descriptions.length);
  });

  it("makes the shelf, each book's contents and each chapter written", () => {
    expect(pages.map(({ folder }) => folder)).toContain("guides/");
    expect(page("guides/developers/quick-start/")?.root).toBe("../../../");
    expect(page("guides/authors/")?.props.book).toBe("authors");
  });

  it("links a chapter's files on GitHub from the chapter's own folder", () => {
    const html = htmlOf("guides/developers/");
    expect(html).toContain(`href="${BLOB}compmd-spec-v1.md"`);
    expect(html).toContain(`href="${BLOB}guides/developers/examples"`);
  });

  it("keeps links to the site on the site the page is read on", () => {
    const html = htmlOf("guides/developers/quick-start/");
    expect(html).toContain('href="../../../playground/?doc=guides/developers/quick-start"');
    expect(html).not.toContain("https://compmd.hakanserce.com/");
  });

  it("links each heading to itself, not to its source on GitHub", () => {
    const html = htmlOf("guides/developers/quick-start/");
    expect(html).toContain('class="anchor"');
    expect(html).not.toContain('class="source"');
  });
});
