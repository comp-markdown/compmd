import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { bookLink, bookPages } from "./book.ts";
import { bookOf, writtenOf } from "./guide.ts";

const GUIDES = join(import.meta.dirname, "../../guides");
const BLOB = "https://github.com/comp-markdown/compmd/blob/main/";
const books = await bookPages((path) => readFileSync(join(GUIDES, path), "utf8"), BLOB);
const developers = books.find(({ book }) => book === "developers");

describe("bookLink", () => {
  it("takes a chapter of the book to its section, and a heading of it to its id there", () => {
    expect(bookLink("developers", "02-quick-start.md")).toBe("#quick-start");
    expect(bookLink("developers", "./09-packs-from-npm.md#the-configuration")).toBe("#packs-from-npm-the-configuration");
  });

  it("takes the book's contents to the top of the book", () => {
    expect(bookLink("developers", "README.md")).toBe("#book");
  });

  it("takes the other book, and the site, to their addresses in full", () => {
    expect(bookLink("developers", "../authors/02-the-document.md")).toBe("https://compmd.hakanserce.com/guides/authors/the-document/");
    expect(bookLink("developers", "../authors/README.md")).toBe("https://compmd.hakanserce.com/guides/authors/");
    expect(bookLink("developers", "https://compmd.hakanserce.com/playground/?doc=x")).toBe("https://compmd.hakanserce.com/playground/?doc=x");
  });

  it("leaves the rest for GitHub", () => {
    expect(bookLink("developers", "../../compmd-spec-v1.md")).toBeUndefined();
  });
});

describe("bookPages", () => {
  it("makes each book a page, its contents and then every chapter written, in order", () => {
    expect(books.map(({ book, folder }) => [book, folder])).toEqual([
      ["authors", "guides/authors/print/"],
      ["developers", "guides/developers/print/"],
    ]);
    const written = writtenOf(bookOf(readFileSync(join(GUIDES, "developers/README.md"), "utf8")));
    expect(developers?.props.chapters).toHaveLength(written.length);
    expect(developers?.props.title).toBe("CompMD Developer's Guide");
    expect(developers?.props.contents).toContain('href="#quick-start"');
  });

  it("gives each chapter its title once, as the heading of its section", () => {
    const first = developers?.props.chapters[0] ?? "";
    expect(first).toMatch(/^<h2 id="introduction">1\. Introduction<a class="anchor"/);
  });
});
