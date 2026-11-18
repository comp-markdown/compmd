// Keys from origins (spec §12.3): the renderer keeps an id per origin range between renders and follows the
// ranges through every edit, as an editor maps positions, so a node whose range lands where an old one went keeps
// its id, and its view. A key is that id with the node's occurrence.
import { walk, type ExpandedNode, type Origin, type TextEdit } from "./tree.ts";

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

/** A processor's edit as the keys follow it. */
export function editOf({ start, end, text }: TextEdit): Edit {
  return { start, oldEnd: end, newEnd: start + codePointOffset(text, text.length), opensLine: text.startsWith("\n") };
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

/** A range as `at` wrote it, or none for an origin without offsets. */
function parsed(where: string): Range | undefined {
  const [start, end] = where.split("-").map(Number);
  return start === undefined || end === undefined || Number.isNaN(start) || Number.isNaN(end) ? undefined : { start, end };
}

/**
 * Where an origin is, `shift` code points on: its range, or, without offsets, its lines and columns, which can't be
 * followed through an edit, so only its own text identifies it.
 */
function whereOf({ start, end }: Origin, shift: number): string {
  if (start.offset !== undefined && end.offset !== undefined) return at({ start: start.offset + shift, end: end.offset + shift });
  return `${String(start.line)}:${String(start.column)}-${String(end.line)}:${String(end.column)}`;
}

/** A block's ids and keys: ids by the ranges its origins show, keys by node. */
export interface Keyed {
  ids: Map<string, number>;
  keys: Map<ExpandedNode, string>;
}

/**
 * The ids of one document's blocks. A block that goes releases its ids; each edit after that moves them, and a new
 * block whose node has a range a released id has after the edits takes that id. Without an edit (an input or data
 * changed), equal ranges keep their ids.
 *
 * @example
 * const keys = new Keys();
 * const heading = keys.assign([node], 0);                    // "# A", at 0-3
 * keys.release(heading.ids, 0);
 * keys.edit({ start: 0, oldEnd: 0, newEnd: 7, opensLine: false }); // "Intro\n\n" typed before it
 * keys.assign([moved], 0).keys.get(moved);                   // at 7-10: the heading's key
 */
export class Keys {
  #next = 0;
  /** The released ids, by range in the current text. */
  #released = new Map<string, number>();
  /** The ids assigned since the last render, by range: blocks an `each` made share their origins. */
  readonly #assigned = new Map<string, number>();

  /** Releases a block's `ids`, by the ranges its origins show, `shift` code points behind the current text. */
  release(ids: ReadonlyMap<string, number>, shift: number): void {
    for (const [where, id] of ids) {
      const range = parsed(where);
      const now = range ? at({ start: range.start + shift, end: range.end + shift }) : where;
      if (!this.#released.has(now)) this.#released.set(now, id);
    }
  }

  /** Moves the released ids through `edit`; those whose ranges it replaced are gone. */
  edit(edit: Edit): void {
    const moved = new Map<string, number>();
    for (const [where, id] of this.#released) {
      const range = parsed(where);
      const now = range ? mapRange(range, edit) : undefined;
      if (now && !moved.has(at(now))) moved.set(at(now), id);
      if (!range && !moved.has(where)) moved.set(where, id);
    }
    this.#released = moved;
  }

  /** The ids and keys of `nodes`, one block's, whose origins are `shift` code points behind the current text. */
  assign(nodes: readonly ExpandedNode[], shift: number): Keyed {
    const keyed: Keyed = { ids: new Map(), keys: new Map() };
    walk(nodes, (node) => {
      if (!node.origin) return;
      const where = whereOf(node.origin, shift);
      const id = this.#released.get(where) ?? this.#assigned.get(where) ?? this.#next++;
      this.#assigned.set(where, id);
      keyed.ids.set(whereOf(node.origin, 0), id);
      keyed.keys.set(node, `${String(id)}#${String(node.origin.occurrence)}`);
    });
    return keyed;
  }

  /** Forgets the released ids no block took: after a render. */
  settle(): void {
    this.#released.clear();
    this.#assigned.clear();
  }
}
