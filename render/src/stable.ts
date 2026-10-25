// What a live page shows while its text is invalid mid-typing. Lenient recovery (spec §10.3) is made for
// a finished document: while someone types, it turns a half-typed param into a fallback (the component unmounts and
// loses its state), a new opener into a container that swallows the rest of the document, and a stray character in
// the frontmatter into a page without components. Three rules on top of it, by key:
//
// 1. Frontmatter that doesn't parse (F001, F033): the last good page stays, and keys aren't reassigned until the text
//    is valid again.
// 2. A top-level container that isn't closed (F004; only the end of the file can close it there): its opener shows
//    as a paragraph, and what follows stays where it was, with its keys and its state.
// 3. Any other error in a top-level block: that block keeps its last good render, found by its key; the rest updates.
import { Keys, utf16Offset, type EditRange, type KeyOf } from "./keys.ts";
import { contentOf, walk, type ExpandedNode, type ExpandedRoot, type Origin } from "./tree.ts";

/** A diagnostic as a processor reports it (spec §10.5): its code and where it was found, 1-based. */
export interface Diagnostic {
  code: string;
  line: number;
  column: number;
}

/** What changed since the last render: the new text, its diagnostics, and the old text's range the editor replaced. */
export interface Change {
  text: string;
  diagnostics?: readonly Diagnostic[];
  edit?: EditRange;
}

/** A tree to render and how to find its nodes' keys. */
export interface Keyed {
  tree: ExpandedRoot;
  keyOf: KeyOf;
}

const WHOLE_DOCUMENT = new Set(["F001", "F033"]);

/** A point as a code-point offset, by its line and column (both count code points, spec §10.5). */
function offsetOf(text: string, line: number, column: number): number {
  let current = 1;
  let offset = 0;
  for (const char of text) {
    if (current === line) break;
    offset++;
    if (char === "\n") current++;
  }
  return offset + column - 1;
}

const rangeOf = (origin: Origin | undefined): { start: number; end: number } | undefined => {
  const start = origin?.start.offset;
  const end = origin?.end.offset;
  return start === undefined || end === undefined ? undefined : { start, end };
};

/**
 * Keys each tree and keeps the last good render of what an invalid edit breaks, by the rules above.
 *
 * @example
 * const stable = new Stable();
 * const { tree, keyOf } = stable.show(processed.tree, { text, diagnostics: processed.diagnostics, edit });
 */
export class Stable {
  readonly #keys: Keys;
  #last: Keyed | undefined;
  /** Whether the last change wasn't keyed (rule 1), so the next edit's range doesn't apply to the keys' text. */
  #skipped = false;

  constructor(keys = new Keys()) {
    this.#keys = keys;
  }

  /** The tree to show for `tree`, processed from `change.text`, with its keys. */
  show(tree: ExpandedRoot, change?: Change): Keyed {
    if (!change) return this.#remember({ tree, keyOf: this.#keys.assign(tree) });
    const errors = (change.diagnostics ?? []).filter((d) => d.code.startsWith("F"));
    if (this.#last && errors.some((d) => WHOLE_DOCUMENT.has(d.code))) {
      this.#skipped = true;
      return this.#last;
    }
    const keyOf = this.#keys.assign(tree, { text: change.text, ...(change.edit && !this.#skipped ? { edit: change.edit } : {}) });
    this.#skipped = false;
    if (errors.length === 0 || !this.#last) return this.#remember({ tree, keyOf });
    return this.#stabilized({ tree, keyOf }, this.#last, errors, change.text);
  }

  #remember(keyed: Keyed): Keyed {
    this.#last = keyed;
    return keyed;
  }

  #stabilized(next: Keyed, last: Keyed, errors: readonly Diagnostic[], text: string): Keyed {
    const lastByKey = new Map<string, ExpandedNode>();
    for (const node of last.tree.children) {
      const key = last.keyOf(node);
      if (key !== undefined) lastByKey.set(key, node);
    }
    const kept = new Set<ExpandedNode>();
    const offsets = errors.map((d) => ({ code: d.code, at: offsetOf(text, d.line, d.column) }));
    const children = next.tree.children.flatMap((node): ExpandedNode[] => {
      const range = rangeOf(node.origin);
      if (!range) return [node];
      const inNode = offsets.filter(({ at }) => at >= range.start && at < Math.max(range.end, range.start + 1));
      if (inNode.length === 0) return [node];
      if (node.type === "componentContainer" && inNode.some(({ code, at }) => code === "F004" && at === range.start)) return asText(node, text, range.start);
      const key = next.keyOf(node);
      const good = key === undefined ? undefined : lastByKey.get(key);
      if (!good) return [node];
      kept.add(good);
      return [good];
    });
    // A kept block is the last tree's, so its nodes' keys are the last ones.
    const inKept = new Set<ExpandedNode>();
    walk([...kept], (n) => inKept.add(n));
    const keyOf: KeyOf = (node) => (inKept.has(node) ? last.keyOf(node) : next.keyOf(node));
    return { tree: { ...next.tree, children }, keyOf };
  }
}

/** An unclosed container as its opener line, in a paragraph with the container's origin, followed by its content. */
function asText(node: ExpandedNode, text: string, start: number): ExpandedNode[] {
  const from = utf16Offset(text, start);
  const lineEnd = text.indexOf("\n", from);
  const opener = text.slice(from, lineEnd === -1 ? text.length : lineEnd);
  const paragraph: ExpandedNode = { type: "paragraph", children: [{ type: "text", value: opener }], ...(node.origin ? { origin: node.origin } : {}) };
  return [paragraph, ...contentOf(node)];
}
