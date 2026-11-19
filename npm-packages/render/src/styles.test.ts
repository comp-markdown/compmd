import { describe, expect, it } from "vitest";
import { blockCss, layoutOf, ownParams, Stylesheet, usesBreakpoints } from "./styles.ts";
import type { ExpandedParam } from "./tree.ts";

const p = (key: string, value: unknown, breakpoint: string | null = null): ExpandedParam => ({ key, breakpoint, value });

describe("blockCss", () => {
  it("writes each param as the CSS property it names, with tokens as custom properties", () => {
    expect(blockCss([p("gap", "m"), p("padding", "s l"), p("width", "40%"), p("background", "subtle"), p("border-radius", "m")], "block", true).base).toEqual({
      gap: "var(--compmd-space-m, 1rem)",
      padding: "var(--compmd-space-s, 0.5rem) var(--compmd-space-l, 2rem)",
      width: "40%",
      background: "var(--compmd-surface-subtle, color-mix(in srgb, currentColor 6%, transparent))",
      "border-radius": "var(--compmd-radius-m, 8px)",
    });
  });

  it("puts breakpoint params at their breakpoint", () => {
    const css = blockCss([p("grid-template-columns", "1fr"), p("grid-template-columns", "3fr 2fr", "md")], "block", true);
    expect(css.base).toEqual({ "grid-template-columns": "1fr" });
    expect(css.at).toEqual({ md: { "grid-template-columns": "3fr 2fr" } });
  });

  it("maps CompMD's own names and values", () => {
    expect(blockCss([p("justify-content", "between"), p("order", "first"), p("aspect-ratio", "none"), p("scroll", true), p("sticky", "top")], "block", true).base).toEqual({
      "justify-content": "space-between",
      order: "-1",
      "aspect-ratio": "auto",
      overflow: "auto",
      position: "sticky",
      "inset-block-start": "0",
    });
  });

  it("makes fill take the height its parent leaves, by how the parent lays it out", () => {
    expect(blockCss([p("fill", true)], "column").base).toEqual({ "flex-grow": "1", "min-block-size": "0" });
    expect(blockCss([p("fill", true)], "row").base).toEqual({ "align-self": "stretch" });
  });

  it("makes justify-self push a block to the end in a flex parent", () => {
    expect(blockCss([p("justify-self", "end")], "grid").base).toEqual({ "justify-self": "end" });
    expect(blockCss([p("justify-self", "end")], "row").base).toEqual({ "margin-inline-start": "auto" });
  });

  it("takes a block with an inset out of the flow at each breakpoint where it has one (spec §9.2)", () => {
    const css = blockCss([p("inset-inline-start", "60%", "md"), p("inset-inline-start", "auto", "lg")], "block");
    expect(css.at.md).toMatchObject({ position: "absolute", "inset-inline-start": "60%" });
    expect(css.at.lg).toMatchObject({ position: "relative" });
  });

  it("keeps show and hide apart, leaves attributes out, and copies no value that could end the rule", () => {
    const css = blockCss([p("show", "md"), p("hide", "lg"), p("class", ["a"]), p("width", "1px; color: red")], "block");
    expect(css).toEqual({ base: {}, at: {}, show: "md", hide: "lg" });
  });

  it("styles only a box by its layout params, and no box by a component's own params", () => {
    expect(blockCss([p("gap", "m"), p("rows", [1]), p("bind", "region")], "block").base).toEqual({});
    expect(blockCss([p("gap", "m")], "block", true).base).toEqual({ gap: "var(--compmd-space-m, 1rem)" });
  });
});

describe("params", () => {
  it("tells a component's own params from its box's", () => {
    expect(ownParams([p("rows", [1]), p("padding", "m"), p("class", ["x"]), p("label", "B", "md")])).toEqual({ rows: [1] });
  });

  it("finds how a box lays out its children", () => {
    expect(layoutOf([p("display", "flex")])).toBe("row");
    expect(layoutOf([p("display", "flex"), p("flex-direction", "column")])).toBe("column");
    expect(layoutOf([p("display", "grid")])).toBe("grid");
    expect(layoutOf([])).toBe("block");
  });

  it("finds params that follow a breakpoint, show and hide among them", () => {
    expect(usesBreakpoints([p("gap", "l", "md")])).toBe(true);
    expect(usesBreakpoints([p("show", "md")])).toBe(true);
    expect(usesBreakpoints([p("gap", "l")])).toBe(false);
  });
});

describe("Stylesheet", () => {
  it("writes base rules, then each breakpoint's as a container query, narrowest first, then visibility", () => {
    const sheet = new Stylesheet(document);
    const a = document.createElement("div");
    const b = document.createElement("div");
    const name = sheet.style(a, blockCss([p("gap", "s"), p("gap", "l", "lg"), p("gap", "m", "sm")], "block", true));
    sheet.style(b, { base: {}, at: {}, show: "md" });
    sheet.flush();
    const text = sheet.text;
    expect(name).toMatch(/^compmd-b\d+$/);
    expect(text).toContain(`.${name} { gap: var(--compmd-space-s, 0.5rem); }`);
    expect(text.indexOf("@container (min-width: 480px)")).toBeLessThan(text.indexOf("@container (min-width: 1024px)"));
    expect(text).toContain("@container (max-width: 767.98px)");
    sheet.forget(a);
    sheet.flush();
    expect(sheet.text).not.toContain(`.${name} `);
    sheet.remove();
  });

  it("trims the margins at a box's edges, below a host's own rules", () => {
    const sheet = new Stylesheet(document);
    sheet.flush();
    expect(sheet.text).toContain(":where(.compmd-box, .compmd-item, .compmd-component) > :first-child { margin-block-start: 0; }");
    expect(sheet.text).toContain(":where(.compmd-box, .compmd-item, .compmd-component) > :last-child { margin-block-end: 0; }");
    sheet.remove();
  });

  it("keeps an element's class across updates", () => {
    const sheet = new Stylesheet(document);
    const element = document.createElement("div");
    expect(sheet.style(element, blockCss([p("gap", "s")], "block"))).toBe(sheet.style(element, blockCss([p("gap", "m")], "block")));
    sheet.remove();
  });
});

describe("a document's stylesheets", () => {
  /** The rules the document has adopted, as text. */
  const adopted = () => document.adoptedStyleSheets.flatMap((sheet) => [...sheet.cssRules]).map((rule) => rule.cssText).join("\n");

  /** Counts the times the document's sheets are set while `run` runs: each restyles the whole document. */
  function adoptions(run: () => void): number {
    const own = Object.getOwnPropertyDescriptor(document, "adoptedStyleSheets");
    let sheets = document.adoptedStyleSheets;
    let count = 0;
    Object.defineProperty(document, "adoptedStyleSheets", {
      configurable: true,
      get: () => sheets,
      set: (next: CSSStyleSheet[]) => {
        count += 1;
        sheets = next;
      },
    });
    try {
      run();
    } finally {
      if (own) Object.defineProperty(document, "adoptedStyleSheets", own);
      else Reflect.deleteProperty(document, "adoptedStyleSheets");
      document.adoptedStyleSheets = sheets;
    }
    return count;
  }

  it("share one sheet, adopted with the first and dropped with the last", () => {
    const before = document.adoptedStyleSheets.length;
    const sheets: Stylesheet[] = [];
    expect(adoptions(() => {
      for (let i = 0; i < 50; i++) sheets.push(new Stylesheet(document));
      for (const sheet of sheets) sheet.flush();
    })).toBe(1);
    expect(document.adoptedStyleSheets).toHaveLength(before + 1);
    expect(adoptions(() => {
      for (const sheet of sheets.slice(1)) sheet.remove();
    })).toBe(0);
    expect(document.adoptedStyleSheets).toHaveLength(before + 1);
    sheets[0]?.remove();
    expect(document.adoptedStyleSheets).toHaveLength(before);
  });

  it("have the base rules once, and each its own blocks' rules until it is removed", () => {
    const [one, two] = [new Stylesheet(document), new Stylesheet(document)];
    const a = one.style(document.createElement("div"), blockCss([p("width", "40%")], "block"));
    const b = two.style(document.createElement("div"), blockCss([p("width", "60%")], "block"));
    one.flush();
    two.flush();
    expect(adopted().split(".compmd-page {").length - 1).toBe(1);
    expect(adopted()).toContain(`.${a} { width: 40%; }`);
    expect(adopted()).toContain(`.${b} { width: 60%; }`);
    expect(one.text).not.toContain(b);
    one.remove();
    expect(adopted()).not.toContain(`.${a} `);
    expect(adopted()).toContain(`.${b} { width: 60%; }`);
    two.remove();
  });

  it("never give two blocks of a document the same class, a removed stylesheet's included", () => {
    const names = new Set<string>();
    for (let i = 0; i < 3; i++) {
      const sheet = new Stylesheet(document);
      names.add(sheet.style(document.createElement("div"), blockCss([p("width", "40%")], "block")));
      names.add(sheet.style(document.createElement("div"), blockCss([p("width", "60%")], "block")));
      sheet.remove();
    }
    expect(names.size).toBe(6);
  });

  it("are a document's own", () => {
    const other = document.implementation.createHTMLDocument();
    const [here, there] = [new Stylesheet(document), new Stylesheet(other)];
    const name = here.style(document.createElement("div"), blockCss([p("width", "40%")], "block"));
    here.flush();
    there.flush();
    expect(adopted()).toContain(`.${name} `);
    expect(there.text).not.toContain(name);
    here.remove();
    there.remove();
  });
});
