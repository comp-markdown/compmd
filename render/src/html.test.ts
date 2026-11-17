import { describe, expect, it } from "vitest";
import { htmlImageSources, ownSchemes, safeUrl, urlScheme, withHtml } from "./html.ts";
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

  it("keeps links and images of the host's own schemes, and only with them", () => {
    const nodes = [html('<a href="app:item/7">'), text("Books"), html('</a><img src="app:file/3" alt="Cover"><img src="other:x" alt="Other">')];
    expect(withHtml(nodes, ["app"])).toMatchObject([
      { tag: "a", attributes: { href: "app:item/7" } },
      { tag: "img", attributes: { src: "app:file/3", alt: "Cover" } },
      { tag: "img", attributes: { alt: "Other" } },
    ]);
    expect(withHtml(nodes)).toMatchObject([{ tag: "a", attributes: {} }, { tag: "img", attributes: { alt: "Cover" } }, { tag: "img", attributes: { alt: "Other" } }]);
    expect(htmlImageSources('<img src="app:file/3"><img src="other:x">', ["app"])).toEqual(["app:file/3"]);
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

  it("allows the host's own schemes too, and only those", () => {
    expect(safeUrl("app:page/1", ["app"])).toBe("app:page/1");
    expect(safeUrl("APP:page/1", ["app"])).toBe("APP:page/1");
    expect(safeUrl("app:page/1")).toBeUndefined();
    expect(safeUrl("other:x", ["app"])).toBeUndefined();
    expect(safeUrl("javascript:x", ["app"])).toBeUndefined();
  });
});

describe("urlScheme", () => {
  it("gives a URL's scheme in lower case, however it is disguised, and none for a relative one", () => {
    expect(urlScheme("App:item/7")).toBe("app");
    expect(urlScheme(" java\tscript:x")).toBe("javascript");
    for (const url of ["./logo.png", "page.md", "#top", "/a"]) expect(urlScheme(url)).toBeUndefined();
  });
});

describe("ownSchemes", () => {
  it("gives the host's schemes as safeUrl takes them", () => {
    expect(ownSchemes(["App", "fig:", "x-my.app+1"])).toEqual(["app", "fig", "x-my.app+1"]);
  });

  it("refuses a scheme that runs code or carries content of its own, and what isn't a scheme", () => {
    for (const scheme of ["javascript", "VBScript", "data:", "blob", "file"]) expect(() => ownSchemes([scheme])).toThrow(/can't be a host's own scheme/);
    for (const scheme of ["", "1app", "app/x", "a b"]) expect(() => ownSchemes([scheme])).toThrow(/isn't a URL scheme/);
  });
});
