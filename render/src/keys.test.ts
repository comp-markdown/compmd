import { describe, expect, it } from "vitest";
import { codePointOffset, diff, Keys, mapRange, utf16Offset, type Edit } from "./keys.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

const insert = (at: number, length: number, opensLine = false): Edit => ({ start: at, oldEnd: at, newEnd: at + length, opensLine });

describe("mapRange", () => {
  it("leaves a range before the edit, and moves one after it", () => {
    expect(mapRange({ start: 0, end: 5 }, insert(10, 3))).toEqual({ start: 0, end: 5 });
    expect(mapRange({ start: 10, end: 20 }, insert(0, 5))).toEqual({ start: 15, end: 25 });
  });

  it("stretches a range over an edit inside it", () => {
    expect(mapRange({ start: 0, end: 10 }, { start: 3, oldEnd: 5, newEnd: 9, opensLine: false })).toEqual({ start: 0, end: 14 });
  });

  it("keeps a start at an insertion with the text after it", () => {
    expect(mapRange({ start: 10, end: 20 }, insert(10, 4))).toEqual({ start: 14, end: 24 });
  });

  it("takes in text typed at a range's end on its line, but not a new line", () => {
    expect(mapRange({ start: 0, end: 10 }, insert(10, 4))).toEqual({ start: 0, end: 14 });
    expect(mapRange({ start: 0, end: 10 }, insert(10, 4, true))).toEqual({ start: 0, end: 10 });
  });

  it("drops a range the edit replaced entirely", () => {
    expect(mapRange({ start: 4, end: 6 }, { start: 2, oldEnd: 8, newEnd: 3, opensLine: false })).toBeUndefined();
  });
});

describe("diff", () => {
  it("finds the edit between two texts, within the editor's range when given", () => {
    expect(diff("a b c", "a X c")).toEqual({ start: 2, oldEnd: 3, newEnd: 3, opensLine: false });
    // Cutting ":: n\n\n" from in front of "::: c": a diff alone would start inside the second opener.
    expect(diff(":: n\n\n::: c", "::: c", { start: 0, end: 6 })).toEqual({ start: 0, oldEnd: 6, newEnd: 0, opensLine: false });
  });

  it("counts code points, as origins do", () => {
    expect(diff("😀a", "😀ba")).toMatchObject({ start: 1, oldEnd: 1, newEnd: 2 });
    expect(codePointOffset("😀a", 2)).toBe(1);
    expect(utf16Offset("😀a", 1)).toBe(2);
  });
});

const node = (start: number, end: number, occurrence = 0): ExpandedNode => ({
  type: "paragraph",
  children: [],
  origin: { start: { line: 1, column: start + 1, offset: start }, end: { line: 1, column: end + 1, offset: end }, occurrence },
});
const root = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });

describe("Keys", () => {
  it("keeps a node's key when an edit before it moves its origin", () => {
    const keys = new Keys();
    const a = node(0, 3);
    const first = keys.assign(root(a), { text: "# A" });
    const b = node(7, 10);
    const second = keys.assign(root(node(0, 5), b), { text: "Intro\n\n# A", edit: { start: 0, end: 0 } });
    expect(second(b)).toBe(first(a));
  });

  it("tells nodes with one origin apart by their occurrence", () => {
    const keys = new Keys();
    const [one, two] = [node(0, 3, 0), node(0, 3, 1)];
    const keyOf = keys.assign(root(one, two));
    expect(keyOf(one)).not.toBe(keyOf(two));
    expect(keyOf(one)?.split("#")[0]).toBe(keyOf(two)?.split("#")[0]);
  });

  it("keeps keys for the same origins without a change (an input changed)", () => {
    const keys = new Keys();
    const before = node(0, 3);
    const first = keys.assign(root(before));
    const again = node(0, 3);
    expect(keys.assign(root(again))(again)).toBe(first(before));
  });

  it("gives a node without an origin no key", () => {
    expect(new Keys().assign(root({ type: "text", value: "x" }))({ type: "text", value: "x" })).toBeUndefined();
  });
});
