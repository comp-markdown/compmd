// The page layer: the region at the root, plus what only the whole tree has: the container breakpoints
// query, the theme tokens, footnote numbering with the list at the end, and the inputs from the tree's root. Given a
// change, it keys the tree through the edit and applies the rules for invalid text (`stable.ts`).
import { definitionsOf, footnotesOf, type Footnotes } from "./document.ts";
import { createRegion, type SurfaceOptions } from "./region.ts";
import { markBox } from "./render.ts";
import { Stable, type Change } from "./stable.ts";
import type { Stylesheet } from "./styles.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

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
  /** Replaces the theme tokens. */
  theme(tokens: Record<string, string>): void;
  destroy(): void;
  /** The document's stylesheet, for regions and components an editor renders inside the page. */
  readonly stylesheet: Stylesheet;
}

const element = (tag: string, attributes: Record<string, string>, content: { text?: string; children?: ExpandedNode[] }): ExpandedNode => ({ type: "compmdElement", tag, attributes, ...content });

/** The footnotes' list, as GitHub shows it: a rule, then each definition with a link back to its first reference. */
function footnotesList({ definitions }: Footnotes): ExpandedNode[] {
  if (definitions.length === 0) return [];
  const items = definitions.map(({ number, children }) =>
    element("li", { id: `compmd-fn-${String(number)}` }, {
      children: [...children, element("a", { href: `#compmd-fnref-${String(number)}`, class: "compmd-footnote-back", "aria-label": `Back to reference ${String(number)}` }, { text: "↩" })],
    }),
  );
  return [element("section", { class: "compmd-footnotes" }, { children: [element("hr", {}, {}), element("ol", {}, { children: items })] })];
}

/**
 * Creates the page in `root`, which it owns: `root` becomes the container breakpoints query, with the theme tokens.
 *
 * @example
 * const page = createPage(document.body, { components: { chart }, onSet: (input, value) => rerender({ [input]: value }) });
 * page.render(tree);                                   // an input changed: same text, same keys
 * page.render(tree, { text, diagnostics, edit });      // a keystroke in an editor
 */
export function createPage(root: HTMLElement, options: PageOptions = {}): Page {
  root.classList.add("compmd-page");
  const region = createRegion(root, options);
  const stable = new Stable();
  let tokens: string[] = [];
  const theme = (next: Record<string, string>) => {
    for (const name of tokens) root.style.removeProperty(name);
    tokens = Object.keys(next).filter((name) => name.startsWith("--compmd-"));
    for (const name of tokens) root.style.setProperty(name, next[name] ?? "");
  };
  theme(options.tokens ?? {});
  return {
    stylesheet: region.stylesheet,
    render(tree, change) {
      const shown = stable.show(tree, change);
      const footnotes = footnotesOf(shown.tree.children);
      const nodes = [...shown.tree.children, ...footnotesList(footnotes)];
      markBox(root, nodes, "block");
      region.render(nodes, {
        ...(shown.tree.inputs ? { inputs: shown.tree.inputs } : {}),
        footnotes: footnotes.numbers,
        definitions: definitionsOf(shown.tree.children),
        keyOf: shown.keyOf,
      });
    },
    theme,
    destroy() {
      region.destroy();
      theme({});
      root.classList.remove("compmd-page", "compmd-container", "compmd-fills");
    },
  };
}
