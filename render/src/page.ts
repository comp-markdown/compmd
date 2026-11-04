// The page layer: the tree's top-level blocks (`blocks.ts`), each rendered once and kept until a change replaces it,
// plus what only the whole tree has: the container breakpoints query, the theme tokens, footnote numbering with the
// list at the end, link definitions, and the inputs from the tree's root. A change is a splice of the blocks, or a
// whole tree; keys follow its edit, and invalid text keeps the last good render (`stable.ts`).
import { Blocks } from "./blocks.ts";
import { diff, editOf, type Edit } from "./keys.ts";
import { Layout } from "./layout.ts";
import { linesIn, type Line } from "./lines.ts";
import { surfaceOf, type SurfaceOptions } from "./region.ts";
import { Schemes } from "./scheme.ts";
import { elementsIn, sourceIn, type Source } from "./source.ts";
import { breaksDocument, errorsOf, type Change } from "./stable.ts";
import { Stylesheet } from "./styles.ts";
import type { ExpandedRoot, Splice } from "./tree.ts";

export interface PageOptions extends SurfaceOptions {
  /** The theme: `--compmd-*` custom properties and their values (spec §9.4). */
  tokens?: Record<string, string>;
}

export interface Page {
  /**
   * Renders `tree`, or brings the page up to date with it. Without `change`, the text is unchanged (an input or data
   * changed). With it, keys follow the edit, and invalid text keeps the last good render (`stable.ts`).
   */
  render(tree: ExpandedRoot, change?: Change): void;
  /**
   * Brings the page up to date with `splices`, in order, each of the tree the one before left, as a processor that
   * keeps the document open gives them: only the blocks they replaced render. Keys follow each splice's edit.
   * `change` has the text after them and its diagnostics, for the rules for invalid text; its `edit` isn't read.
   */
  splice(splices: readonly Splice[], change?: Change): void;
  /** Replaces the theme tokens. */
  theme(tokens: Record<string, string>): void;
  /**
   * Reads the page's colour scheme again, after the host switched its own theme: components follow it. It is read at
   * each render too, when the reader's preference changes, and when printing starts and ends.
   */
  scheme(): void;
  destroy(): void;
  /** The document's stylesheet, for regions and components an editor renders inside the page. */
  readonly stylesheet: Stylesheet;
  /**
   * The range of the source that made `node`, a node of the page's DOM (an element or text, in a component's shadow
   * root too): its own expanded node's origin, or that of the nearest one around it that has one, in the current
   * text. What the renderer adds itself (a footnote's ↩) has its parent's range; a component's own DOM, the
   * component's. `undefined` outside the page's blocks, or when the tree has no origins (request them with
   * `origins: true`; spec §12.3).
   *
   * @param node - Where the reader clicked or the selection is: an event's `target`, a selection's `anchorNode`.
   * @returns Lines and columns 1-based, offsets 0-based, columns and offsets in code points.
   *
   * @example
   * root.addEventListener("click", (event) => {
   *   const source = page.sourceOf(event.target as Node);
   *   if (source?.start.offset !== undefined) editor.moveCaret(utf16Offset(text, source.start.offset));
   * });
   */
  sourceOf(node: Node): Source | undefined;
  /**
   * The elements that the source at `offset` drew: those of the innermost node whose range holds it. Where one range
   * ends and the next starts, the next. Several when its node drew several (each iteration of an `each`); of elements
   * drawn inside one another for one range (a `compmd` component's box and its body), the outermost. Empty when nothing
   * the page shows came from there (a definition fence, the frontmatter), or when the tree has no origins.
   *
   * @param offset - A code-point offset in the current text; an editor's UTF-16 offset through `codePointOffset`.
   *
   * @example
   * editor.onCaret((caret) => page.elementsAt(codePointOffset(text, caret))[0]?.scrollIntoView({ block: "nearest" }));
   */
  elementsAt(offset: number): Element[];
  /**
   * Each line of the current text that draws something, in line order from line `from`, with the boxes it shows in,
   * one per visual line it covers, in CSS pixels from the top left corner of the root's content: where its text shows,
   * a code block's line in its `<pre>`, or, for a line with no text of its own (a component's, an image's), the box of
   * what its range drew. Lines that draw nothing (blank lines, frontmatter, a closing line) are left out. Each line is
   * measured when the generator reaches it; `onLayout` says when to ask again.
   *
   * @param from - The first line to give, 1-based; 1 without it.
   *
   * @example
   * for (const { line, rects: [first] } of page.lines(firstShown)) {
   *   if (first.y > root.scrollTop + root.clientHeight) break;   // past what shows: the rest isn't measured
   *   gutter.append(number(line, first.y));
   * }
   */
  lines(from?: number): Generator<Line>;
}

/**
 * Creates the page in `root`, which it owns: `root` becomes the container breakpoints query, with the theme tokens.
 *
 * @example
 * const page = createPage(document.body, { components: { chart }, onSet: (input, value) => rerender({ [input]: value }) });
 * page.render(tree);                                   // an input changed: same text, same keys
 * page.render(tree, { text, diagnostics, edit });      // a keystroke in an editor
 * page.splice([splice], { text, diagnostics });        // a keystroke, with a document kept open
 * page.elementsAt(42);                                 // what the source at offset 42 drew
 */
export function createPage(root: HTMLElement, options: PageOptions = {}): Page {
  root.classList.add("compmd-page");
  const stylesheet = new Stylesheet(root.ownerDocument);
  const schemes = new Schemes(root);
  const surface = surfaceOf(root.ownerDocument, stylesheet, schemes, options);
  const blocks = new Blocks(root, surface);
  const onLayout = options.onLayout?.bind(options);
  const layout = onLayout ? new Layout(root, onLayout) : undefined;
  /** The text the blocks are of, as the last change gave it. */
  let text: string | undefined;
  let rendered = false;
  const show = (change: Change | undefined) => {
    if (change) text = change.text;
    const errors = errorsOf(change);
    // Rule 1: the blocks follow the change, and show when the frontmatter parses again.
    if (rendered && breaksDocument(errors)) return;
    blocks.render(errors, text);
    stylesheet.flush();
    surface.pending.run();
    rendered = true;
    layout?.rendered();
  };
  let tokens: string[] = [];
  const theme = (next: Record<string, string>) => {
    for (const name of tokens) root.style.removeProperty(name);
    tokens = Object.keys(next).filter((name) => name.startsWith("--compmd-"));
    for (const name of tokens) root.style.setProperty(name, next[name] ?? "");
  };
  theme(options.tokens ?? {});
  return {
    stylesheet,
    render(tree, change) {
      schemes.read();
      const edit: Edit | undefined = change && text !== undefined && change.text !== text ? diff(text, change.text, change.edit) : undefined;
      blocks.apply({ from: 0, to: blocks.length, nodes: tree.children, lines: 0, offset: 0, ...(tree.inputs ? { inputs: tree.inputs } : {}) }, edit);
      show(change);
    },
    splice(splices, change) {
      schemes.read();
      for (const splice of splices) blocks.apply(splice, splice.edit ? editOf(splice.edit) : undefined);
      show(change);
    },
    theme,
    scheme() {
      schemes.read();
    },
    sourceOf: (node) => sourceIn(blocks.placed(), node, root),
    elementsAt: (offset) => elementsIn(blocks.placed(), offset),
    lines: (from) => linesIn(blocks.placed(), root, from),
    destroy() {
      layout?.destroy();
      schemes.destroy();
      blocks.destroy();
      stylesheet.remove();
      theme({});
      root.classList.remove("compmd-page", "compmd-container", "compmd-fills");
    },
  };
}
