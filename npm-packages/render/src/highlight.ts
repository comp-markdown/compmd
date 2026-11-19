// Highlighting a code block that shows as code: a highlighter says what each line's tokens are, and the renderer
// draws them into the block's own lines (`code-lines.ts`). So a highlighter never touches the page, and how a block
// is drawn (its lines, their numbers) stays the renderer's own.
import { CODE_LINE } from "./code-lines.ts";

/** A token of a highlighted line: a piece of its text, and how it shows. */
export interface CodeToken {
  /** The token's text, without a line break. */
  text: string;
  /** Its colour in each colour scheme, as CSS colours; without it, the text's own colour. */
  color?: { light: string; dark: string };
  italic?: boolean;
  bold?: boolean;
  underline?: boolean;
}

/** A highlighted block: each line's tokens, whose texts joined are the line. */
export type CodeLines = readonly (readonly CodeToken[])[];

/**
 * Highlights a code block that shows as code: gives the tokens of `code`, written in the fence language `language`
 * (`""` for none), line by line, or `undefined` for a language it doesn't highlight. It may give them later, as a
 * promise, once what it needs has loaded: the block shows its text until then, and keeps it when the promise is
 * rejected. It is called again only when the block's text or language changes.
 *
 * The tokens must be the block's text: each line's texts joined give the line, and the lines are the code's. Tokens
 * that aren't are dropped, and the block keeps its text.
 *
 * @example
 * const highlight: CodeHighlighter = (code, language) =>
 *   language === "todo" ? code.split("\n").map((line) => [{ text: line, bold: line.startsWith("!") }]) : undefined;
 * createPage(root, { highlight });
 */
export type CodeHighlighter = (code: string, language: string) => CodeLines | undefined | Promise<CodeLines | undefined>;

/** What each block asked for last, so tokens that come after a newer text's are dropped. */
const latest = new WeakMap<HTMLElement, object>();

/**
 * Highlights `target`, a block's `<code>` holding `code` as its lines, with `highlight`: its tokens drawn at once when
 * it gives them at once, or when they come, unless the block has changed since.
 */
export function highlightCode(target: HTMLElement, code: string, language: string, highlight: CodeHighlighter): void {
  const asked = {};
  latest.set(target, asked);
  let given: ReturnType<CodeHighlighter>;
  try {
    given = highlight(code, language);
  } catch {
    return;
  }
  if (!(given instanceof Promise)) {
    if (given) drawTokens(target, code, given);
    return;
  }
  void given.then(
    (lines) => {
      if (lines && latest.get(target) === asked) drawTokens(target, code, lines);
    },
    () => undefined,
  );
}

/**
 * Puts `lines` in `target` as spans, each line's in its line's element, which keeps its attributes and its line break,
 * unless they aren't `code`'s text: then the block keeps its text.
 */
function drawTokens(target: HTMLElement, code: string, lines: CodeLines): void {
  if (lines.map((line) => line.map(({ text }) => text).join("")).join("\n") !== code) return;
  const elements = [...target.children].filter((child) => child.classList.contains(CODE_LINE));
  if (elements.length !== lines.length) return;
  const document = target.ownerDocument;
  for (const [i, line] of lines.entries()) {
    const parts: Node[] = line.map(({ text, color, italic, bold, underline }) => {
      const span = document.createElement("span");
      span.textContent = text;
      // Through the CSSOM, which a frame's policy allows where it refuses a `style` attribute; it follows the
      // `color-scheme` of the page around it, so nothing is drawn again when that changes.
      if (color) span.style.setProperty("color", `light-dark(${color.light}, ${color.dark})`);
      if (italic) span.style.setProperty("font-style", "italic");
      if (bold) span.style.setProperty("font-weight", "bold");
      if (underline) span.style.setProperty("text-decoration", "underline");
      return span;
    });
    if (i < lines.length - 1) parts.push(document.createTextNode("\n"));
    elements[i]?.replaceChildren(...parts);
  }
}
