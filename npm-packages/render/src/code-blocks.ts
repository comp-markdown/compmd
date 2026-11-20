// A document's code blocks, for a page that can't highlight them itself: a page in a frame asks its host for each
// block's tokens, and the host answers only for the blocks of the trees it sent.
import { textOf, walk, type ExpandedNode } from "./tree.ts";

/** A code block as a highlighter is asked for it: its text and its fence's language (`""` for none). */
export interface CodeBlock {
  code: string;
  language: string;
}

/**
 * Every code block `nodes` hold, slots and branches shown included, as the renderer asks a `CodeHighlighter` for it.
 *
 * @example
 * codeBlocks(tree.children); // [{ code: "let a = 1;", language: "ts" }]
 */
export function codeBlocks(nodes: readonly ExpandedNode[]): CodeBlock[] {
  const blocks: CodeBlock[] = [];
  walk(nodes, (node) => {
    if (node.type === "code") blocks.push({ code: textOf(node.value), language: textOf(node.lang) });
  });
  return blocks;
}
