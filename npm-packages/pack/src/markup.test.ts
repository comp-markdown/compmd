// A drawing's markup made into nodes and a stylesheet, as `draw` shows it in a page that refuses inline styles.
import { describe, expect, it } from "vitest";
import { DrawingSheet, parseMarkup } from "./markup.ts";

// HTML: happy-dom's parser leaves out an SVG's children, which Mermaid's page test draws in real browsers.
const MARKUP = '<div id="d1" style="max-width: 120px"><style>#d1 .node { color: teal; }</style><p class="node"><span style="width: 10px; height: 4px">A</span></p></div>';

describe("parseMarkup", () => {
  it("gives the markup's CSS apart, and its style attributes as the elements' styles", () => {
    const { nodes, css } = parseMarkup(MARKUP, document);
    expect(css).toBe("#d1 .node { color: teal; }");
    const drawing = nodes.firstElementChild as HTMLElement;
    expect(drawing.ownerDocument).toBe(document);
    expect(drawing.querySelector("style")).toBeNull();
    expect([drawing.getAttribute("style"), drawing.style.maxWidth]).toEqual(["max-width: 120px;", "120px"]);
    const span = drawing.querySelector("span") as HTMLElement;
    expect([span.style.width, span.style.height]).toEqual(["10px", "4px"]);
  });

  it("keeps markup with neither as it is", () => {
    const { nodes, css } = parseMarkup("<b>drawn</b> and text", document);
    expect(css).toBe("");
    expect(nodes.textContent).toBe("drawn and text");
  });
});

describe("DrawingSheet", () => {
  it("adopts the drawing's CSS on the document it is shown in, in place of the last, and drops it", () => {
    const target = document.body.appendChild(document.createElement("div"));
    const before = document.adoptedStyleSheets.length;
    const sheet = new DrawingSheet(target);
    sheet.replace(".a { color: red; }", target);
    sheet.replace(".b { color: blue; }", target);
    expect(document.adoptedStyleSheets.length).toBe(before + 1);
    expect(document.adoptedStyleSheets.at(-1)?.cssRules[0]?.cssText).toContain(".b");
    sheet.replace("  ", target);
    expect(document.adoptedStyleSheets.length).toBe(before);
    sheet.replace(".c { color: green; }", target);
    sheet.drop();
    expect(document.adoptedStyleSheets.length).toBe(before);
    target.remove();
  });

  it("adopts it on a shadow root the drawing is shown in", () => {
    const host = document.body.appendChild(document.createElement("div"));
    const shadow = host.attachShadow({ mode: "open" });
    const target = shadow.appendChild(document.createElement("div"));
    const sheet = new DrawingSheet(target);
    sheet.replace(".a { color: red; }", target);
    expect(shadow.adoptedStyleSheets.length).toBe(1);
    sheet.drop();
    expect(shadow.adoptedStyleSheets.length).toBe(0);
    host.remove();
  });
});
