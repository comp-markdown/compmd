import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { guidePages } from "./guide-pages.ts";

const GUIDES = join(import.meta.dirname, "../../guides");
const BLOB = "https://github.com/comp-markdown/compmd/blob/main/";
const pages = await guidePages((path) => readFileSync(join(GUIDES, path), "utf8"), BLOB);
const page = (folder: string) => pages.find((candidate) => candidate.folder === folder);
const htmlOf = (folder: string): string => String(page(folder)?.props.html);

describe("guidePages", () => {
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
});
