import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { isRelative, rebased, rebasedTree } from "./rebase.ts";

const image = (url: string, pack?: boolean): ExpandedNode => ({ type: "image", url, alt: "", ...(pack ? { pack } : {}) });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });

describe("isRelative and rebased", () => {
  it("rebase a URL relative to the document, and keep the others", () => {
    expect(["photos/a.png", "./a.png", "../a.png"].map(isRelative)).toEqual([true, true, true]);
    expect(["https://a.example/b.png", "/b.png", "#top", "?q=1", "mailto:a@example.com", ""].map(isRelative)).toEqual([false, false, false, false, false, false]);
    expect(rebased("./photos/a.png", "../")).toBe("../photos/a.png");
    expect(rebased("#top", "../")).toBe("#top");
  });
});

describe("rebasedTree", () => {
  it("rebases the post's images, links, link definitions and fonts' files, in its examples too", () => {
    const tree: ExpandedRoot = {
      type: "root",
      fonts: [{ family: "serif", fallback: "body", files: [{ url: "fonts/a.woff2", weight: "normal", style: "normal" }, { url: "kit/b.woff2", pack: true, weight: "bold", style: "normal" }] }],
      children: [
        paragraph(image("photos/a.png"), { type: "link", url: "other-post/", children: [] }, { type: "link", url: "#top", children: [] }),
        { type: "definition", identifier: "a", url: "notes.pdf" },
        { type: "code", lang: "compmd", meta: null, value: "", render: "output", document: { type: "root", children: [paragraph(image("photos/b.png"))] } },
      ],
    };
    const rebasedOne = rebasedTree(tree, "../");
    expect(JSON.stringify(rebasedOne)).toContain('"url":"../photos/a.png"');
    expect(JSON.stringify(rebasedOne)).toContain('"url":"../other-post/"');
    expect(JSON.stringify(rebasedOne)).toContain('"url":"#top"');
    expect(JSON.stringify(rebasedOne)).toContain('"url":"../notes.pdf"');
    expect(JSON.stringify(rebasedOne)).toContain('"url":"../photos/b.png"');
    expect(rebasedOne.fonts?.[0]?.files.map(({ url }) => url)).toEqual(["../fonts/a.woff2", "kit/b.woff2"]);
    expect(JSON.stringify(tree)).toContain('"url":"photos/a.png"');
  });

  it("keeps a pack's images, and params, which are data", () => {
    const tree: ExpandedRoot = {
      type: "root",
      children: [paragraph(image("@kit/a@1/star.svg", true)), { type: "componentLeaf", name: "gallery", params: [{ key: "items", breakpoint: null, value: [{ type: "image", url: "photos/a.png" }] }] }],
    };
    expect(rebasedTree(tree, "../")).toEqual(tree);
  });

  it("is the tree itself with nothing to put before its URLs", () => {
    const tree: ExpandedRoot = { type: "root", children: [paragraph(image("a.png"))] };
    expect(rebasedTree(tree, "")).toBe(tree);
  });
});
