// The page layer: the tree's top-level blocks (`blocks.ts`), each rendered once and kept until a change replaces it,
// plus what only the whole tree has: the container breakpoints query, the theme tokens, footnote numbering with the
// list at the end, link definitions, and the inputs from the tree's root. A change is a splice of the blocks, or a
// whole tree; keys follow its edit, and invalid text keeps the last good render (`stable.ts`).
import { Blocks } from "./blocks.ts";
import { diff, editOf, type Edit } from "./keys.ts";
import { surfaceOf, type SurfaceOptions } from "./region.ts";
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
  destroy(): void;
  /** The document's stylesheet, for regions and components an editor renders inside the page. */
  readonly stylesheet: Stylesheet;
}

/**
 * Creates the page in `root`, which it owns: `root` becomes the container breakpoints query, with the theme tokens.
 *
 * @example
 * const page = createPage(document.body, { components: { chart }, onSet: (input, value) => rerender({ [input]: value }) });
 * page.render(tree);                                   // an input changed: same text, same keys
 * page.render(tree, { text, diagnostics, edit });      // a keystroke in an editor
 * page.splice([splice], { text, diagnostics });        // a keystroke, with a document kept open
 */
export function createPage(root: HTMLElement, options: PageOptions = {}): Page {
  root.classList.add("compmd-page");
  const stylesheet = new Stylesheet(root.ownerDocument);
  const blocks = new Blocks(root, surfaceOf(root.ownerDocument, stylesheet, options));
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
    rendered = true;
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
      const edit: Edit | undefined = change && text !== undefined && change.text !== text ? diff(text, change.text, change.edit) : undefined;
      blocks.apply({ from: 0, to: blocks.length, nodes: tree.children, lines: 0, offset: 0, ...(tree.inputs ? { inputs: tree.inputs } : {}) }, edit);
      show(change);
    },
    splice(splices, change) {
      for (const splice of splices) blocks.apply(splice, splice.edit ? editOf(splice.edit) : undefined);
      show(change);
    },
    theme,
    destroy() {
      blocks.destroy();
      stylesheet.remove();
      theme({});
      root.classList.remove("compmd-page", "compmd-container", "compmd-fills");
    },
  };
}
