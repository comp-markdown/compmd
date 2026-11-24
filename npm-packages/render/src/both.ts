// A block that shows its code and what it renders (`render=both`, spec §7.7, §7.8), built in one place for a code
// renderer's block and an example (audit R4): the code alone, as a plain code block, then the output's tag.

import { outputTag } from "./code-frame.ts";
import type { View } from "./render.ts";
import type { ExpandedCode, ExpandedNode, MarkdownNode } from "./tree.ts";

/** A code block's code alone: its language, its info and its text, without what renders it. */
export function plainCode(node: ExpandedNode): MarkdownNode {
  const { lang, meta, value } = node as ExpandedCode;
  return { type: "code", lang, meta, value };
}

/** Puts `code`, the block's code shown folded, and the output's tag in `box`, marked as `render=both`'s. */
export function showBoth(box: HTMLElement, code: View): void {
  box.classList.add("compmd-both");
  box.append(code.dom, outputTag(box.ownerDocument));
}
