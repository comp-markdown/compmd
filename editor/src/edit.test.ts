import { describe, expect, it } from "vitest";
import { editOf, fixOf, offsetOf, type Edit } from "./edit.ts";

/** `before` with `edit` applied. */
const applied = (before: string, { start, end, text }: Edit) => before.slice(0, start) + text + before.slice(end);

describe("editOf", () => {
  const cases: [name: string, before: string, after: string, selection: [number, number], expected: Edit][] = [
    ["typing", "Hello world", "Hello world!", [11, 11], { start: 11, end: 11, text: "!" }],
    ["Backspace", "Hello world", "Hello worl", [11, 11], { start: 10, end: 11, text: "" }],
    ["Delete", "Hello world", "Hello orld", [6, 6], { start: 6, end: 7, text: "" }],
    ["a selection pasted over", "Hello world", "Hello there", [6, 11], { start: 6, end: 11, text: "there" }],
    ["a selection deleted", "Hello world", "Hello ", [6, 11], { start: 6, end: 11, text: "" }],
    [
      "a cut that leaves what follows looking the same",
      ":: notes[Notes]\n::: card\n",
      "::: card\n",
      [0, 16],
      { start: 0, end: 16, text: "" },
    ],
    ["a paste at the start", "Hello", "One.\n\nTwo.\n\nHello", [0, 0], { start: 0, end: 0, text: "One.\n\nTwo.\n\n" }],
    ["undo, away from the caret", "abc def", "abc", [7, 7], { start: 3, end: 7, text: "" }],
    ["the same character typed over itself", "abc", "abc", [1, 2], { start: 1, end: 2, text: "b" }],
    ["one emoji for another", "a😀b", "a😁b", [1, 3], { start: 1, end: 3, text: "😁" }],
    ["an emoji's second half differing, with the caret after it", "😀", "😁", [2, 2], { start: 0, end: 2, text: "😁" }],
  ];

  for (const [name, before, after, [start, end], expected] of cases) {
    it(name, () => {
      const edit = editOf(before, after, { start, end });
      expect(edit).toEqual(expected);
      expect(applied(before, edit)).toBe(after);
    });
  }
});

describe("fixOf", () => {
  const range = (line: number, from: number, to: number) => ({ start: { line, column: from }, end: { line, column: to } });

  it("places a fix by its line and code-point columns", () => {
    const text = "# 😀 Title\n::: box {border=subtl}\n";
    const edit = fixOf(text, { range: range(2, 17, 22), text: "subtle" });
    expect(applied(text, edit)).toBe("# 😀 Title\n::: box {border=subtle}\n");
    expect(fixOf("😀ab", { range: range(1, 2, 3), text: "x" })).toEqual({ start: 2, end: 3, text: "x" });
  });

  it("counts after a byte order mark, and stops at the end", () => {
    expect(offsetOf("﻿ab", { line: 1, column: 2 })).toBe(2);
    expect(offsetOf("ab", { line: 5, column: 1 })).toBe(2);
    expect(offsetOf("ab", { line: 1, column: 9 })).toBe(2);
  });
});
