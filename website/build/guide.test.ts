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
    ]);
    expect(writtenOf(bookOf(CONTENTS)).map((chapter) => chapter.slug)).toEqual(["introduction", "quick-start"]);
  });

  it("reads the guide's own contents", () => {
    const book = bookOf(readFileSync(join(import.meta.dirname, "../../guide/README.md"), "utf8"));
    expect(book.length).toBeGreaterThan(1);
    expect(writtenOf(book)[0]?.slug).toBe("introduction");
  });
});

describe("guideLink", () => {
  it("takes a chapter to its page, and the contents to the guide's", () => {
    expect(guideLink("02-quick-start.md", "../")).toBe("../quick-start/");
    expect(guideLink("./03-how-it-fits-together.md#why", "")).toBe("how-it-fits-together/#why");
    expect(guideLink("README.md#contents", "../")).toBe("../#contents");
    expect(guideLink("README.md", "")).toBe("./");
  });

  it("leaves anything else to GitHub", () => {
    expect(guideLink("../wasm/README.md", "../")).toBeUndefined();
    expect(guideLink("examples", "")).toBeUndefined();
  });
});

describe("slugOf", () => {
  it("drops the number and the extension", () => {
    expect(slugOf("12-your-first-pack.md")).toBe("your-first-pack");
  });
});
