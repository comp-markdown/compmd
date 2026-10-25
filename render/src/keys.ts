// Keys from origins (spec §12.3): the renderer keeps an id per origin range between renders and follows the
// ranges through every edit, as an editor maps positions, so a node whose range lands where an old one went keeps
// its id, and its view. A key is that id with the node's occurrence.
import { walk, type ExpandedNode, type ExpandedRoot, type Origin } from "./tree.ts";

/** The range of the old text an edit replaced, as an editor reports it: UTF-16 offsets, as in a JavaScript string. */
export interface EditRange {
  start: number;
  end: number;
}

/** An edit in code points (spec §10.5): `[start, oldEnd)` of the old text became `[start, newEnd)`. */
export interface Edit {
  start: number;
  oldEnd: number;
  newEnd: number;
  /** Whether the new text starts a new line; text typed at a block's end on its line belongs to the block. */
  opensLine: boolean;
}

/** A range of the document, in code points. */
export interface Range {
  start: number;
  end: number;
}

/** The code-point offset of a UTF-16 offset in `text`. */
export function codePointOffset(text: string, utf16: number): number {
  let offset = 0;
  for (let i = 0; i < utf16 && i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code < 0xdc00 || code > 0xdfff) offset++;
  }
  return offset;
}

/** The UTF-16 offset of a code-point offset in `text`. */
export function utf16Offset(text: string, codePoint: number): number {
  let i = 0;
  for (let seen = 0; seen < codePoint && i < text.length; seen++) i += (text.codePointAt(i) ?? 0) > 0xffff ? 2 : 1;
  return i;
}

/**
 * The edit that turns `before` into `after`, inside `within` (the old text's range the editor changed) when given,
 * otherwise from the texts' common prefix and suffix. A diff alone can misplace an edit: cutting `:: notes[Notes]`
 * from in front of `::: card` leaves `::` where it was, so it starts inside the card's opener.
 *
 * @example
 * diff("a b c", "a X c"); // { start: 2, oldEnd: 3, newEnd: 3, opensLine: false }
 */
export function diff(before: string, after: string, within?: EditRange): Edit {
  const window = within ?? { start: 0, end: before.length };
  const old = before.slice(window.start, window.end);
  const replacement = after.slice(window.start, after.length - (before.length - window.end));
  let prefix = 0;
  while (prefix < Math.min(old.length, replacement.length) && old[prefix] === replacement[prefix]) prefix++;
  let suffix = 0;
  while (suffix < Math.min(old.length, replacement.length) - prefix && old[old.length - 1 - suffix] === replacement[replacement.length - 1 - suffix]) suffix++;
  const start = window.start + prefix;
  const oldEnd = window.end - suffix;
  const newEnd = window.start + replacement.length - suffix;
  return {
    start: codePointOffset(before, start),
    oldEnd: codePointOffset(before, oldEnd),
    newEnd: codePointOffset(after, newEnd),
    opensLine: after.charAt(start) === "\n",
  };
}

/**
 * Where `range` is after `edit`: unchanged before it, moved after it, stretched over it; gone when the edit replaced
 * all of it. A start at an insertion stays with the text after it, so text typed right before a block isn't part of
 * it. An end takes in text typed right at it on its line (finishing a half-typed component line keeps the
 * component), but not text that starts a new line.
 *
 * @example
 * mapRange({ start: 10, end: 20 }, { start: 0, oldEnd: 0, newEnd: 5, opensLine: false }); // { start: 15, end: 25 }
 */
export function mapRange(range: Range, edit: Edit): Range | undefined {
  const delta = edit.newEnd - edit.oldEnd;
  const insertion = edit.oldEnd === edit.start;
  const map = (offset: number, staysAtInsertion: boolean): number | undefined => {
    if (offset < edit.start || (offset === edit.start && (!insertion || staysAtInsertion))) return offset;
    if (offset >= edit.oldEnd) return offset + delta;
    return undefined;
  };
  const start = map(range.start, false);
  const end = map(range.end, edit.opensLine);
  if (start === undefined && end === undefined) return undefined;
  return { start: start ?? edit.start, end: end ?? edit.newEnd };
}

/** Finds the key of a node of the tree the keys were assigned to. */
export type KeyOf = (node: ExpandedNode) => string | undefined;

const at = (range: Range): string => `${String(range.start)}-${String(range.end)}`;

/** Code-point offsets of a 1-based line and column in `text`. */
function pointOffset(lines: readonly number[], line: number, column: number): number {
  return (lines[line - 1] ?? lines.at(-1) ?? 0) + column - 1;
}

/** The code-point offset where each line of `text` starts. */
function lineStarts(text: string): number[] {
  const starts = [0];
  let offset = 0;
  for (const char of text) {
    offset++;
    if (char === "\n") starts.push(offset);
  }
  return starts;
}

/**
 * An origin's range in code points: its offsets, or its lines and columns when it has no offsets and the text is
 * known. Without either it can't be followed through an edit, and only its own text identifies it.
 */
function rangeOf(origin: Origin, lines: (() => number[]) | undefined): Range | string {
  const { start, end } = origin;
  if (start.offset !== undefined && end.offset !== undefined) return { start: start.offset, end: end.offset };
  if (!lines) return `${String(start.line)}:${String(start.column)}-${String(end.line)}:${String(end.column)}`;
  return { start: pointOffset(lines(), start.line, start.column), end: pointOffset(lines(), end.line, end.column) };
}

/**
 * The keys of successive trees of one document. Without a change, the text is the one before (an input or data
 * changed), so equal origins keep their keys. With one, the old origins are followed through its edit first.
 *
 * @example
 * const keys = new Keys();
 * keys.assign(first, { text: "# A\n" });
 * const keyOf = keys.assign(second, { text: "Intro\n\n# A\n", edit: { start: 0, end: 0 } }); // the heading keeps its key
 */
export class Keys {
  #ids = new Map<string, number>();
  #text: string | undefined;
  #next = 0;

  assign(tree: ExpandedRoot, change?: { text: string; edit?: EditRange | undefined }): KeyOf {
    const moved = this.#moved(change);
    const text = change?.text ?? this.#text;
    let lines: number[] | undefined;
    const linesOf = text === undefined ? undefined : () => (lines ??= lineStarts(text));
    const ids = new Map<string, number>();
    const keys = new Map<ExpandedNode, string>();
    walk(tree.children, (node) => {
      if (!node.origin) return;
      const range = rangeOf(node.origin, linesOf);
      const where = typeof range === "string" ? range : at(range);
      const id = moved.get(where) ?? ids.get(where) ?? this.#next++;
      ids.set(where, id);
      keys.set(node, `${String(id)}#${String(node.origin.occurrence)}`);
    });
    this.#ids = ids;
    this.#text = text;
    return (node) => keys.get(node);
  }

  /** The last ids, by the range each one's origin has after `change`'s edit. */
  #moved(change: { text: string; edit?: EditRange | undefined } | undefined): Map<string, number> {
    if (!change || this.#text === undefined || change.text === this.#text) return this.#ids;
    const edit = diff(this.#text, change.text, change.edit);
    const moved = new Map<string, number>();
    for (const [where, id] of this.#ids) {
      const [start, end] = where.split("-").map(Number);
      if (start === undefined || end === undefined || Number.isNaN(start) || Number.isNaN(end)) continue;
      const range = mapRange({ start, end }, edit);
      if (range && !moved.has(at(range))) moved.set(at(range), id);
    }
    return moved;
  }
}
