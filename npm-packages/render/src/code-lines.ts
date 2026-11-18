// A code block's lines, each an element of its own, so a host can style one (a diff's two sides kept level, a line
// marked): `<span class="compmd-line" data-line="N">`, N its line in the current text, its newline inside it, so the
// block's text, and what a reader copies, are the code's. The renderer draws them, numbers them again when what is
// before the block moves them, and a highlighter fills them with its tokens.
import type { Placed } from "./source.ts";

/** The class of a code block's line. */
export const CODE_LINE = "compmd-line";

/** How many lines `text` has: one more than its line breaks. */
export const lineCount = (text: string): number => text.split("\n").length;

/**
 * `code`'s text, `text`, drawn in `code` as one line element per line: each line's text, then its line break but the
 * last's. A blank line is an element with its line break alone.
 */
export function drawCodeLines(code: HTMLElement, text: string): void {
  const document = code.ownerDocument;
  const lines = text.split("\n");
  code.replaceChildren(
    ...lines.map((line, i) => {
      const element = document.createElement("span");
      element.className = CODE_LINE;
      element.textContent = i < lines.length - 1 ? `${line}\n` : line;
      return element;
    }),
  );
}

/**
 * The `<code>` of a code block `placed` drew, with the line its code starts on: a fenced block's starts on the line
 * after its fence, an indented one's on its first line. None for anything else, or for a kept render, an older
 * text's, whose lines aren't the current text's.
 */
export function codeOf({ dom, source, node, kept }: Placed): { code: Element; first: number } | undefined {
  if (kept || node.type !== "code" || dom.nodeName !== "PRE") return undefined;
  const code = (dom as Element).firstElementChild;
  if (!code) return undefined;
  const { start, end } = source;
  const fenced = end.line - start.line + 1 > lineCount(code.textContent);
  return { code, first: start.line + (fenced ? 1 : 0) };
}

/** Numbers the lines of each code block in `placed` with their lines in the current text, `data-line`. */
export function numberCodeLines(placed: Iterable<Placed>): void {
  for (const each of placed) {
    const found = codeOf(each);
    if (!found) continue;
    let line = found.first;
    for (const child of found.code.children) {
      if (!child.classList.contains(CODE_LINE)) continue;
      const number = String(line++);
      if (child.getAttribute("data-line") !== number) child.setAttribute("data-line", number);
    }
  }
}
