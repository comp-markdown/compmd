import { describe, expect, it } from "vitest";
import { entriesOf, isBranch, leafText, summaryOf } from "./json.ts";

describe("the JSON tree", () => {
  it("folds objects and arrays only", () => {
    expect([isBranch({}), isBranch([]), isBranch(null), isBranch("x"), isBranch(1)]).toEqual([true, true, false, false, false]);
  });

  it("lists entries by key or index", () => {
    expect(entriesOf(["a", "b"])).toEqual([
      ["0", "a"],
      ["1", "b"],
    ]);
    expect(entriesOf({ a: 1 })).toEqual([["a", 1]]);
  });

  it("sums up a folded branch", () => {
    expect(summaryOf([1, 2, 3])).toBe("[3]");
    expect(summaryOf({ type: "heading", depth: 2, children: [] })).toBe("heading (depth 2)");
    expect(summaryOf({ type: "componentLeaf", name: "chip", params: [] })).toBe("componentLeaf (name chip)");
    expect(summaryOf({ type: "paragraph" })).toBe("paragraph");
    expect(summaryOf({ month: "2026-01", units: 4 })).toBe('{month: "2026-01", units: 4}');
    expect(summaryOf({ a: 1, b: 2, c: 3, d: 4 })).toBe("{a: 1, b: 2, c: 3, …}");
    expect(summaryOf({ a: {}, b: 2 })).toBe("{b: 2, …}");
    expect(summaryOf({ a: {} })).toBe("{1 field}");
    expect(summaryOf({ a: [], b: {} })).toBe("{2 fields}");
  });

  it("writes leaves as JSON does", () => {
    expect([leafText("a"), leafText(1), leafText(null), leafText(true), leafText(undefined)]).toEqual(['"a"', "1", "null", "true", "undefined"]);
  });
});
