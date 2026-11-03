import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { bookOf, guideLink, slugOf, writtenOf } from "./guide.ts";

const CONTENTS = `# The guide

Words.

## Contents

**Part I. Getting started**

1. [Introduction](01-introduction.md)
2. [Quick start](02-quick-start.md)

**Part II. Documents** *(to come)*

3. Processing documents

**Appendices**

4. [The diagnostics](04-the-diagnostics.md)

## The examples

1. [Not a chapter](examples/x.md)
`;

describe("bookOf", () => {
  it("reads the parts and the chapters, written or not", () => {
    expect(bookOf(CONTENTS)).toEqual([
      {
        title: "Part I. Getting started",
        later: false,
        chapters: [
          { number: 1, title: "Introduction", file: "01-introduction.md", slug: "introduction" },
          { number: 2, title: "Quick start", file: "02-quick-start.md", slug: "quick-start" },
        ],
      },
      { title: "Part II. Documents", later: true, chapters: [{ number: 3, title: "Processing documents" }] },
      { title: "Appendices", later: false, chapters: [{ number: 4, title: "The diagnostics", file: "04-the-diagnostics.md", slug: "the-diagnostics" }] },
    ]);
    expect(writtenOf(bookOf(CONTENTS)).map((chapter) => chapter.slug)).toEqual(["introduction", "quick-start", "the-diagnostics"]);
  });

  it("reads the guides' own contents", () => {
    const read = (book: string) => bookOf(readFileSync(join(import.meta.dirname, `../../guides/${book}/README.md`), "utf8"));
    expect(writtenOf(read("developers"))[0]?.slug).toBe("introduction");
    expect(read("authors").length).toBeGreaterThan(1);
  });
});

describe("guideLink", () => {
  it("takes a chapter to its page, and the contents to its book's", () => {
    expect(guideLink("02-quick-start.md", "../")).toBe("../quick-start/");
    expect(guideLink("./03-how-it-fits-together.md#why", "")).toBe("how-it-fits-together/#why");
    expect(guideLink("README.md#contents", "../")).toBe("../#contents");
    expect(guideLink("README.md", "")).toBe("./");
  });

  it("takes a link to another book to its pages", () => {
    expect(guideLink("../authors/README.md", "")).toBe("../authors/");
    expect(guideLink("../authors/README.md", "../")).toBe("../../authors/");
    expect(guideLink("../developers/02-quick-start.md", "../")).toBe("../../developers/quick-start/");
  });

  it("takes the shelf's links to each book's pages", () => {
    expect(guideLink("authors/README.md", undefined)).toBe("authors/");
    expect(guideLink("developers/01-introduction.md", undefined)).toBe("developers/introduction/");
  });

  it("leaves anything else to GitHub", () => {
    expect(guideLink("../../wasm/README.md", "../")).toBeUndefined();
    expect(guideLink("examples", "")).toBeUndefined();
    expect(guideLink("../compmd-spec-v1.md", undefined)).toBeUndefined();
  });
});

describe("slugOf", () => {
  it("drops the number and the extension", () => {
    expect(slugOf("12-your-first-pack.md")).toBe("your-first-pack");
  });
});
