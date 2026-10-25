import { describe, expect, it } from "vitest";
import { safeUrl, withHtml } from "./html.ts";
import type { ExpandedNode } from "./tree.ts";

const html = (value: string): ExpandedNode => ({ type: "html", value });
const text = (value: string): ExpandedNode => ({ type: "text", value });

describe("withHtml", () => {
  it("pairs an allowed tag across the Markdown between its halves", () => {
    expect(withHtml([html("<kbd>"), text("Ctrl"), html("</kbd>")])).toEqual([{ type: "compmdHtml", tag: "kbd", attributes: {}, children: [text("Ctrl")] }]);
  });

  it("pairs block HTML around paragraphs, several tags to a node", () => {
    const paragraph: ExpandedNode = { type: "paragraph", children: [text("Hidden")] };
    const [details] = withHtml([html("<details>\n<summary>More</summary>"), paragraph, html("</details>")]);
    expect(details).toMatchObject({ tag: "details", children: [text("\n"), { tag: "summary", children: [text("More")] }, paragraph] });
  });

  it("keeps only allowed attributes, with safe URLs", () => {
    expect(withHtml([html('<a href="https://x.dev" onclick="evil()">'), text("x"), html("</a>")])[0]).toMatchObject({ attributes: { href: "https://x.dev" } });
    expect(withHtml([html('<a href="javascript:alert(1)">')])[0]).toMatchObject({ attributes: {} });
    expect(withHtml([html('<img src="a.png" alt="A" style="x">')])[0]).toMatchObject({ tag: "img", attributes: { src: "a.png", alt: "A" }, children: [] });
  });

  it("shows tags outside the allowlist, and tags that don't pair, as text; drops comments", () => {
    expect(withHtml([html("<script>alert(1)</script>")])).toEqual([text("<script>"), text("alert(1)"), text("</script>")]);
    expect(withHtml([html("</sub>"), html("<!-- note -->")])).toEqual([text("</sub>")]);
  });

  it("returns a list without HTML as it is", () => {
    const nodes = [text("a")];
    expect(withHtml(nodes)).toBe(nodes);
  });
});

describe("safeUrl", () => {
  it("allows relative URLs, fragments, http, https and mailto", () => {
    for (const url of ["page.md", "#top", "/a", "http://a", "https://a", "mailto:a@b.c"]) expect(safeUrl(url)).toBe(url);
  });

  it("refuses other schemes, however they are disguised", () => {
    for (const url of ["javascript:x", "JavaScript:x", " java\tscript:x", "data:text/html,x", "vbscript:x"]) expect(safeUrl(url)).toBeUndefined();
  });
});
