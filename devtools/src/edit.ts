// What a text area's input changed, as the processor and the renderer's keys take it: the range of the old text, in
// UTF-16 positions as the text area counts them, and what replaced it.

/** `[start, end)` of the old text became `text`. */
export interface Edit {
  start: number;
  end: number;
  text: string;
}

/** A range of the text, as `selectionStart` and `selectionEnd` give it. */
export interface Selection {
  start: number;
  end: number;
}

/**
 * The edit that turned `before` into `after`, where `selection` is what was selected before the input. When the text
 * outside the selection is unchanged (typing, pasting, cutting), the selection is what was replaced: cutting
 * `:: notes[Notes]` in front of `::: card` leaves `::` where it was, and a diff alone would place the edit inside the
 * card's opener. Otherwise (Backspace, Delete, undo, which change text next to the caret or anywhere) the edit covers
 * the selection and what changed outside it.
 *
 * @example
 * editOf("abc", "ac", { start: 2, end: 2 }); // Backspace after `b`: { start: 1, end: 2, text: "" }
 */
export function editOf(before: string, after: string, selection: Selection): Edit {
  const prefix = commonPrefix(before, after, Math.min(before.length, after.length));
  const kept = before.length - selection.end;
  if (prefix >= selection.start && commonSuffix(before, after, Math.min(kept, after.length - selection.start)) === kept) {
    return { ...selection, text: after.slice(selection.start, after.length - kept) };
  }
  const suffix = commonSuffix(before, after, Math.min(before.length, after.length) - prefix);
  let start = Math.min(prefix, selection.start);
  let end = Math.max(before.length - suffix, selection.end);
  // Whole characters: 😀 and 😁 share their first unit, so the texts can differ in the middle of one.
  if (isLowSurrogate(before, start)) start--;
  if (isLowSurrogate(before, end)) end++;
  return { start, end, text: after.slice(start, after.length - (before.length - end)) };
}

/** A place as the processor gives it: 1-based line, 1-based column in code points (spec §10.5). */
export interface Place {
  line: number;
  column: number;
}

/** Where `place` is in `text`, in UTF-16 units; columns count after a leading byte order mark, as the processor's. */
export function offsetOf(text: string, { line, column }: Place): number {
  let at = text.startsWith("﻿") ? 1 : 0;
  for (let n = 1; n < line; n++) {
    const next = text.indexOf("\n", at);
    if (next === -1) return text.length;
    at = next + 1;
  }
  for (let c = 1; c < column && at < text.length; c++) at += (text.codePointAt(at) ?? 0) > 0xffff ? 2 : 1;
  return at;
}

/**
 * The edit a diagnostic's fix makes in `text`.
 *
 * @example
 * fixOf("::: box {border=subtl}", { range: { start: { line: 1, column: 17 }, end: { line: 1, column: 22 } }, text: "subtle" });
 * // { start: 16, end: 21, text: "subtle" }
 */
export function fixOf(text: string, fix: { range: { start: Place; end: Place }; text: string }): Edit {
  return { start: offsetOf(text, fix.range.start), end: offsetOf(text, fix.range.end), text: fix.text };
}

/** How many units `a` and `b` start with in common, at most `most`. */
function commonPrefix(a: string, b: string, most: number): number {
  let length = 0;
  while (length < most && a.charCodeAt(length) === b.charCodeAt(length)) length++;
  return length;
}

/** How many units `a` and `b` end with in common, at most `most`. */
function commonSuffix(a: string, b: string, most: number): number {
  let length = 0;
  while (length < most && a.charCodeAt(a.length - 1 - length) === b.charCodeAt(b.length - 1 - length)) length++;
  return length;
}

/** Whether `text`'s unit at `index` is the second half of a character. */
function isLowSurrogate(text: string, index: number): boolean {
  const unit = text.charCodeAt(index);
  return unit >= 0xdc00 && unit <= 0xdfff;
}
