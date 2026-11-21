// The generated signature fence: short forms, the long form for a doc, slots, and the box's defaults.
import { describe, expect, it } from "vitest";
import type { Wrap } from "./config.ts";
import type { Param } from "./map.ts";
import { signatureFence, writtenValue } from "./signature.ts";

const wrap: Wrap = { from: "qr", export: "default", kind: "react", params: [] };
const param = (name: string, type: string, more: Partial<Param> = {}): Param => ({ name, prop: name, type, optional: false, ...more });

describe("signatureFence", () => {
  it("writes each param in the short form, optional or with its default, and the long form for one with a doc", () => {
    const params = [param("value", "string"), param("level", "L | M | H | Q", { optional: true }), param("margin", "number", { optional: true, default: 4 }), param("x", "string", { doc: "The field" })];
    expect(signatureFence("qr-code", params, wrap)).toBe(
      "```compmd component=qr-code signature\nparams:\n  value: string\n  level: L | M | H | Q?\n  margin: number = 4\n  x:\n    type: string\n    doc: The field\n```",
    );
  });

  it("names the slots, and puts the box's defaults on the fence", () => {
    const fence = signatureFence("card", [], { ...wrap, slots: { default: "children" }, box: " width=12rem max-width=100% " });
    expect(fence).toBe("```compmd component=card signature {width=12rem max-width=100%}\nparams: {}\nslots:\n  - default\n```");
  });
});

describe("writtenValue", () => {
  it("writes a default bare when it can, else quoted or as JSON", () => {
    expect(["L", "two words", 4, true, [1], { a: 1 }].map(writtenValue)).toEqual(["L", '"two words"', "4", "true", "[1]", '{"a":1}']);
  });
});
