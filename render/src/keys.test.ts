import { describe, expect, it } from "vitest";
import { codePointOffset, diff, editOf, Keys, mapRange, utf16Offset, type Edit } from "./keys.ts";
import type { ExpandedNode } from "./tree.ts";

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

describe("editOf", () => {
  it("counts the inserted text in code points, and sees a new line", () => {
    expect(editOf({ start: 2, end: 4, text: "😀b" })).toEqual({ start: 2, oldEnd: 4, newEnd: 4, opensLine: false });
    expect(editOf({ start: 0, end: 0, text: "\nx" })).toMatchObject({ newEnd: 2, opensLine: true });
  });
});

const node = (start: number, end: number, occurrence = 0): ExpandedNode => ({
  type: "paragraph",
  children: [],
  origin: { start: { line: 1, column: start + 1, offset: start }, end: { line: 1, column: end + 1, offset: end }, occurrence },
});

describe("Keys", () => {
  it("keeps a node's key when an edit before it moves its origin", () => {
    const keys = new Keys();
    const a = node(0, 3);
    const first = keys.assign([a], 0);
    keys.release(first.ids, 0);
    keys.edit(insert(0, 7));
    const b = node(7, 10);
    expect(keys.assign([b], 0).keys.get(b)).toBe(first.keys.get(a));
  });

  it("follows a block's ids from where its origins were, `shift` behind", () => {
    const keys = new Keys();
    const a = node(0, 3);
    const first = keys.assign([a], 0);
    // Blocks before it grew by 5 since it was rendered: its origins say 0-3, it is at 5-8.
    keys.release(first.ids, 5);
    keys.edit(insert(8, 1));
    const b = node(5, 9);
    expect(keys.assign([b], 0).keys.get(b)).toBe(first.keys.get(a));
    keys.settle();
    expect(keys.assign([node(5, 9)], 0).keys.get(b)).toBeUndefined();
  });

  it("tells nodes with one origin apart by their occurrence", () => {
    const [one, two] = [node(0, 3, 0), node(0, 3, 1)];
    const { keys } = new Keys().assign([one, two], 0);
    expect(keys.get(one)).not.toBe(keys.get(two));
    expect(keys.get(one)?.split("#")[0]).toBe(keys.get(two)?.split("#")[0]);
  });

  it("keeps keys for the same origins without an edit (an input changed)", () => {
    const keys = new Keys();
    const before = node(0, 3);
    const first = keys.assign([before], 0);
    keys.release(first.ids, 0);
    const again = node(0, 3);
    expect(keys.assign([again], 0).keys.get(again)).toBe(first.keys.get(before));
  });

  it("drops the ids of ranges an edit replaced", () => {
    const keys = new Keys();
    const a = node(2, 4);
    keys.release(keys.assign([a], 0).ids, 0);
    keys.edit({ start: 0, oldEnd: 6, newEnd: 1, opensLine: false });
    const b = node(0, 1);
    expect(keys.assign([b], 0).keys.get(b)).toBe("1#0");
  });

  it("keeps an origin without offsets only by its own lines and columns", () => {
    const keys = new Keys();
    const lines: ExpandedNode = { type: "paragraph", origin: { start: { line: 2, column: 1 }, end: { line: 2, column: 4 }, occurrence: 0 } };
    const first = keys.assign([lines], 0);
    keys.release(first.ids, 3);
    keys.edit(insert(0, 1));
    expect(keys.assign([lines], 0).keys.get(lines)).toBe(first.keys.get(lines));
  });

  it("gives a node without an origin no key", () => {
    const text: ExpandedNode = { type: "text", value: "x" };
    expect(new Keys().assign([text], 0).keys.get(text)).toBeUndefined();
  });
});
