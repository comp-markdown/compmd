// @vitest-environment node
import { describe, expect, it } from "vitest";
import { prerendered, relocated } from "./prerender.ts";

const html = '<head><link rel="stylesheet" crossorigin href="./assets/index-1.css"><!--head--></head><body><div id="app"><!--app--></div></body>';

describe("prerendered", () => {
  it("puts the rendered head and body in place, and inlines the stylesheet", () => {
    const read = (href: string) => (href === "./assets/index-1.css" ? "p{color:red}" : "");
    expect(prerendered(html, { head: "<title>T</title>", body: "<p>$& $1</p>" }, read)).toBe(
      '<head><style>p{color:red}</style><title>T</title></head><body><div id="app"><p>$& $1</p></div></body>',
    );
  });

  it("inlines a stylesheet a folder up, and leaves one elsewhere linked", () => {
    const page = '<link rel="stylesheet" href="../assets/s.css"><link rel="stylesheet" href="https://example.org/a.css"><!--head--><!--app-->';
    expect(prerendered(page, { head: "", body: "" }, (href) => `/*${href}*/`)).toBe('<style>/*../assets/s.css*/</style><link rel="stylesheet" href="https://example.org/a.css">');
  });

  it("refuses a page without a marker", () => {
    expect(() => prerendered("<body></body>", { head: "", body: "" }, () => "")).toThrow("the page has no <!--head-->");
  });
});

describe("relocated", () => {
  it("moves paths up from one folder down to the page's own depth", () => {
    const page = '<link rel="icon" href="../logo/icon.svg"><script src="../assets/spec.js"></script><a href="https://x.org/../a">';
    expect(relocated(page, "../../")).toBe('<link rel="icon" href="../../logo/icon.svg"><script src="../../assets/spec.js"></script><a href="https://x.org/../a">');
    expect(relocated(page, "../")).toBe(page);
  });
});
