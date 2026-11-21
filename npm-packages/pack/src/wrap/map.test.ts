// A prop as a param (decision 6 of docs/wrapping-an-npm-package.md): each kind of type, what is left out and why, the
// overrides, and inputs.
import { describe, expect, it } from "vitest";
import type { Wrap } from "./config.ts";
import { defaultValue, mapProp } from "./map.ts";
import type { Prop, PropType } from "./props.ts";

const wrap: Wrap = { from: "qr", export: "default", kind: "react", params: [] };
const prop = (name: string, type: PropType, more: Partial<Prop> = {}): Prop => ({ name, type, optional: true, ...more });

describe("mapProp", () => {
  it("maps each kind of type the spec has", () => {
    const mapped = (type: PropType) => {
      const param = mapProp(prop("p", type), wrap);
      return "type" in param ? param.type : param.why;
    };
    expect(["string", "number", "boolean", "date", "array", "object", "color"].map((kind) => mapped({ kind } as PropType))).toEqual(["string", "number", "boolean", "date", "array", "object", "color"]);
    expect(mapped({ kind: "literals", values: ["L", "M", "Q", "H"] })).toBe("L | M | Q | H");
    expect(mapped({ kind: "literals", values: ["on top", "below"] })).toBe("string");
  });

  it("takes a union's one usable branch, saying so, and leaves out one with none or several", () => {
    expect(mapProp(prop("title", { kind: "union", branches: [{ kind: "string" }, { kind: "function" }] }), wrap)).toMatchObject({ type: "string", note: "its string branch, of 2" });
    expect(mapProp(prop("any", { kind: "union", branches: [{ kind: "string" }, { kind: "number" }] }), wrap)).toEqual({ prop: "any", why: "a union of string, number: name its type in `overrides`" });
    expect(mapProp(prop("odd", { kind: "union", branches: [{ kind: "function" }, { kind: "other", text: "symbol" }] }), wrap)).toMatchObject({ why: "a union with no branch a param can be" });
  });

  it("leaves out callbacks, what the box takes, React's and the DOM's own, and a size in pixels", () => {
    const why = (name: string, type: PropType = { kind: "string" }) => {
      const mapped = mapProp(prop(name, type), wrap);
      return "why" in mapped ? mapped.why : undefined;
    };
    expect(why("onDone", { kind: "function" })).toBe("a callback");
    expect(why("height", { kind: "number" })).toMatch(/common param/);
    expect(why("class")).toMatch(/common param/);
    expect(why("style")).toBe("the box's to style");
    expect(why("ref")).toBe("React's own");
    expect(why("size", { kind: "number" })).toMatch(/a size in pixels/);
    expect(why("size", { kind: "literals", values: ["s", "l"] })).toBeUndefined();
    expect(why("symbol", { kind: "other", text: "symbol" })).toBe("its type, symbol, is no param's");
  });

  it("keeps a prop's doc, optional mark and JSDoc default, the overrides over them", () => {
    const level = prop("level", { kind: "literals", values: ["L", "H"] }, { doc: "Loss", default: '"L"' });
    expect(mapProp(level, wrap)).toEqual({ name: "level", prop: "level", type: "L | H", optional: true, default: "L", doc: "Loss" });
    const overrides = { ...wrap, overrides: { fgColor: { name: "color", default: "black" }, x: { type: "string", doc: "The field" } } };
    expect(mapProp(prop("fgColor", { kind: "color" }), overrides)).toEqual({ name: "color", prop: "fgColor", type: "color", optional: true, default: "black" });
    expect(mapProp(prop("x", { kind: "function" }), overrides)).toMatchObject({ name: "x", type: "string", doc: "The field" });
  });

  it("makes an input of a prop the overrides name, with no default, and leaves out its callback and a slot's prop", () => {
    const inputs = { ...wrap, input: { value: "onChange", tone: "onTone" }, slots: { default: "children" } };
    expect(mapProp(prop("value", { kind: "string" }, { default: '"a"' }), inputs)).toEqual({ name: "value", prop: "value", type: "input<string>", optional: true });
    expect(mapProp(prop("tone", { kind: "literals", values: ["a", "b"] }), inputs)).toMatchObject({ type: "input<enum>" });
    expect(mapProp(prop("onChange", { kind: "function" }), inputs)).toEqual({ prop: "onChange", why: "reports the input `value`" });
    expect(mapProp(prop("children", { kind: "object" }), inputs)).toEqual({ prop: "children", why: "the slot `default`" });
    expect(mapProp(prop("children", { kind: "object" }), wrap)).toMatchObject({ why: /name it in `slots`/ });
  });
});

describe("defaultValue", () => {
  it("reads a default as JSON, a quoted string, or as written", () => {
    expect(["256", "true", '"L"', "'M'", "`Q`", "[1, 2]", "auto"].map(defaultValue)).toEqual([256, true, "L", "M", "Q", [1, 2], "auto"]);
  });
});
