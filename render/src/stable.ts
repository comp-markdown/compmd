// What a live page shows while its text is invalid mid-typing. Lenient recovery (spec §10.3) is made for
// a finished document: while someone types, it turns a half-typed param into a fallback (the component unmounts and
// loses its state), a new opener into a container that swallows the rest of the document, and a stray character in
// the frontmatter into a page without components. Three rules on top of it, by key:
//
// 1. Frontmatter that doesn't parse (F001, F033): the last good page stays until the text is valid again.
// 2. A top-level container that isn't closed (F004; only the end of the file can close it there): its opener shows
//    as a paragraph, and what follows stays where it was, with its keys and its state.
// 3. Any other error in a top-level block: that block keeps its last good render, found by its key; the rest updates.
//
// The page applies rule 1; a block it renders shows what rules 2 and 3 say.
import { utf16Offset, type EditRange } from "./keys.ts";
import { contentOf, walk, type ExpandedNode, type Point } from "./tree.ts";

/** A diagnostic as a processor reports it (spec §10.5): its code and where it was found, 1-based. */
export interface Diagnostic {
  code: string;
  line: number;
  column: number;
}

/**
 * What changed since the last render: the new text, its diagnostics, and, for a whole tree, the old text's range the
 * editor replaced.
 */
export interface Change {
  text: string;
  diagnostics?: readonly Diagnostic[];
  edit?: EditRange;
}

/** A block's last good render: its node, and the keys of that node's tree. */
export interface Good {
  node: ExpandedNode;
  keys: ReadonlyMap<ExpandedNode, string>;
}

/** What rules 2 and 3 need: the errors, the text they are in, and the last good renders of blocks that went. */
export interface Rules {
  errors: readonly Diagnostic[];
  text: string | undefined;
  good: ReadonlyMap<string, Good>;
}

const WHOLE_DOCUMENT = new Set(["F001", "F033"]);

/** The errors of `change`, which the rules read. */
export const errorsOf = (change: Change | undefined): Diagnostic[] => (change?.diagnostics ?? []).filter((d) => d.code.startsWith("F"));

/** Whether `errors` break the whole document (rule 1). */
export const breaksDocument = (errors: readonly Diagnostic[]): boolean => errors.some((d) => WHOLE_DOCUMENT.has(d.code));

const before = (a: { line: number; column: number }, b: Point): boolean => a.line < b.line || (a.line === b.line && a.column < b.column);
const at = (a: { line: number; column: number }, b: Point): boolean => a.line === b.line && a.column === b.column;

/** `point`, `lines` lines on. */
const moved = (point: Point, lines: number): Point => ({ ...point, line: point.line + lines });

/** A point as a code-point offset, by its line and column (both count code points, spec §10.5). */
function offsetOf(text: string, { line, column }: Point): number {
  let current = 1;
  let offset = 0;
  for (const char of text) {
    if (current === line) break;
    offset++;
    if (char === "\n") current++;
  }
  return offset + column - 1;
}

/**
 * What a block shows: its node, or what rules 2 and 3 say for an error in it. Its origins are `lines` lines and
 * `shift` code points behind the text; `keys` are its node's tree's, to which a kept render's are added.
 */
export function shownOf(node: ExpandedNode, place: { lines: number; shift: number }, keys: Map<ExpandedNode, string>, rules: Rules): ExpandedNode[] {
  const origin = node.origin;
  if (!origin || rules.errors.length === 0) return [node];
  const start = moved(origin.start, place.lines);
  const end = moved(origin.end, place.lines);
  const inNode = rules.errors.filter((d) => !before(d, start) && (before(d, end) || at(d, start)));
  if (inNode.length === 0) return [node];
  if (node.type === "componentContainer" && rules.text !== undefined && inNode.some((d) => d.code === "F004" && at(d, start))) {
    return asText(node, rules.text, origin.start.offset === undefined ? offsetOf(rules.text, start) : origin.start.offset + place.shift);
  }
  const key = keys.get(node);
  const good = key === undefined ? undefined : rules.good.get(key);
  if (!good) return [node];
  // A kept render is the last tree's, so its nodes' keys are the last ones.
  walk([good.node], (n) => {
    const kept = good.keys.get(n);
    if (kept !== undefined) keys.set(n, kept);
  });
  return [good.node];
}

/** An unclosed container as its opener line, in a paragraph with the container's origin, followed by its content. */
function asText(node: ExpandedNode, text: string, start: number): ExpandedNode[] {
  const from = utf16Offset(text, start);
  const lineEnd = text.indexOf("\n", from);
  const opener = text.slice(from, lineEnd === -1 ? text.length : lineEnd);
  const paragraph: ExpandedNode = { type: "paragraph", children: [{ type: "text", value: opener }], ...(node.origin ? { origin: node.origin } : {}) };
  return [paragraph, ...contentOf(node)];
}
