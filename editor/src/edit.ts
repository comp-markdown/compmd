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
