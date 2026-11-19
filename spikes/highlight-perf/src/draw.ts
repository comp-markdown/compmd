// A block's tokens drawn into its lines, as `@compmd/sandbox`'s highlighter drew them when the spike was run, and as
// `@compmd/render` draws a highlighter's tokens since (CMD-251): each line's tokens in the renderer's element for that
// line, with their colours as `light-dark()`. The spike keeps its own, so its variants say when a block is drawn and
// which blocks are on screen.

import type { Lines } from "./shiki.ts";

/** The class of a code block's line, the renderer's own. */
const CODE_LINE = "compmd-line";

const ITALIC = 1;
const BOLD = 2;
const UNDERLINE = 4;

/** Draws `lines` into `target`'s line elements; false when they aren't `code`'s text or its lines. */
export function draw(lines: Lines, code: string, target: HTMLElement): boolean {
  if (lines.map((line) => line.map(({ content }) => content).join("")).join("\n") !== code) return false;
  const elements = [...target.children].filter((child) => child.classList.contains(CODE_LINE));
  if (elements.length !== lines.length) return false;
  const document = target.ownerDocument;
  for (const [i, line] of lines.entries()) {
    const parts: Node[] = line.map(({ content, variants }) => {
      const span = document.createElement("span");
      span.textContent = content;
      const { light, dark } = variants;
      if (light?.color && dark?.color) span.style.setProperty("color", `light-dark(${light.color}, ${dark.color})`);
      const style = light?.fontStyle ?? 0;
      if (style & ITALIC) span.style.setProperty("font-style", "italic");
      if (style & BOLD) span.style.setProperty("font-weight", "bold");
      if (style & UNDERLINE) span.style.setProperty("text-decoration", "underline");
      return span;
    });
    if (i < lines.length - 1) parts.push(document.createTextNode("\n"));
    elements[i]?.replaceChildren(...parts);
  }
  return true;
}
