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

  it("keeps an element's class across updates", () => {
    const sheet = new Stylesheet(document);
    const element = document.createElement("div");
    expect(sheet.style(element, blockCss([p("gap", "s")], "block"))).toBe(sheet.style(element, blockCss([p("gap", "m")], "block")));
    sheet.remove();
  });
});
