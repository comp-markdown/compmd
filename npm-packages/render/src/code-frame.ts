// A code block's frame: a bar over its `<pre>` naming its language and its lines, with a button that copies the code,
// and, for a `render=both` block's code (spec §7.7), one that shows it, folded on screen until the reader asks for it.
// The bar's words come from its attributes through the base rules' CSS, so a page's text and what a reader selects
// stay the document's. One listener on the document handles every frame's buttons (`followCodeFrames`), so a frame in
// HTML written ahead, with no renderer on the page, works once the page calls it.
import { lineCount } from "./code-lines.ts";

/** The class of a code block's frame. */
export const CODE_FRAME = "compmd-code";

/** The class of a frame whose code is folded. */
export const FOLDED = "compmd-folded";

const COPY = "compmd-code-copy";
const TOGGLE = "compmd-code-toggle";

/** How long Copy says it copied, in milliseconds. */
const COPIED_FOR = 1500;

/**
 * The frame's rules: the bar a row, its words from its attributes (the language, the lines, the buttons'), a folded
 * frame's code, Copy and output tag hidden on screen, the tag hanging over the output's corner, taking no room; on
 * paper, the code without the bar or the tag. Every document the renderer renders in has them (`styles.ts`); a page of
 * HTML written ahead adopts them itself. A host styles the rest, and can word the buttons in its own language.
 */
export const CODE_FRAME_RULES = `
:where(.compmd-both) { --compmd-both-gap: var(--compmd-space-s, 0.5rem); gap: var(--compmd-both-gap); }
.compmd-code-bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5em; }
.compmd-code-bar::before { content: attr(data-language); order: -2; }
.compmd-code-bar::after { content: attr(data-lines); order: -1; margin-inline-end: auto; }
.compmd-code-copy::before { content: "Copy"; }
.compmd-code-copy[data-copied]::before { content: "Copied"; }
.compmd-code-toggle::before { content: "Show code"; }
.compmd-code-toggle[aria-expanded="true"]::before { content: "Hide code"; }
.compmd-both-tag { display: flex; justify-content: flex-end; align-items: flex-start; block-size: 0; position: relative; z-index: 1; }
.compmd-both-tag > span::before { content: "Output"; }
@media screen {
  .compmd-folded > pre, .compmd-folded .compmd-code-copy, .compmd-folded + .compmd-both-tag { display: none; }
  .compmd-code:not(.compmd-folded) + .compmd-both-tag + * { margin-block-start: calc(-1 * var(--compmd-both-gap, 0px)); }
}
@media print {
  .compmd-code-bar, .compmd-both-tag { display: none; }
}
`;

let made = 0;

/**
 * The frame of the code block `pre`.
 *
 * @example
 * const frame = new CodeFrame(pre); // in pre's place, holding it
 * frame.show("js", "const a = 1;"); // the bar reads "js · 1 line"
 */
export class CodeFrame {
  /** The frame, which holds the bar and `pre`. */
  readonly dom: HTMLElement;
  readonly #bar: HTMLElement;

  /**
   * Puts `pre` in a frame, in its place if it has one; with `folds`, the frame has a button that shows the code, folded
   * until the reader asks for it.
   */
  constructor(pre: HTMLElement, folds = false) {
    const document = pre.ownerDocument;
    followCodeFrames(document);
    this.dom = document.createElement("div");
    this.dom.className = CODE_FRAME;
    pre.replaceWith(this.dom);
    this.#bar = this.dom.appendChild(document.createElement("div"));
    this.#bar.className = "compmd-code-bar";
    this.#bar.appendChild(button(document, COPY, "Copy the code"));
    if (folds) {
      pre.id ||= `compmd-code-${String(++made)}`;
      const toggle = this.#bar.appendChild(button(document, TOGGLE, ""));
      toggle.setAttribute("aria-controls", pre.id);
      fold(this.dom, toggle, true);
    }
    this.dom.appendChild(pre);
  }

  /** Names the code: its fence `language` and the lines of its `text`. */
  show(language: string, text: string): void {
    const lines = lineCount(text);
    setData(this.#bar, "language", language);
    setData(this.#bar, "lines", lines === 1 ? "1 line" : `${String(lines)} lines`);
  }
}

/**
 * The tag that names a `both` block's output while its code shows, to go after the code's frame: its word, "Output",
 * comes from the base rules' CSS.
 */
export function outputTag(document: Document): HTMLElement {
  const tag = document.createElement("div");
  tag.className = "compmd-both-tag";
  tag.appendChild(document.createElement("span"));
  return tag;
}

/**
 * The mark of a document whose frames are followed, on the document itself and by a registered symbol, so two copies
 * of the renderer in one page (a site's script and a live post's) add one listener between them, not one each, which
 * would toggle a frame twice for each click.
 */
const FOLLOWED = Symbol.for("compmd.codeFrames");

/**
 * Handles the buttons of every code frame in `document`, now and later: Copy copies its frame's code, or selects it
 * where the page may not write the clipboard; Show code shows its frame's code, and folds it again. Once per document;
 * the renderer calls it for the documents it renders in, and a page of HTML written ahead calls it for its own.
 */
export function followCodeFrames(document: Document): void {
  const marked = document as Document & { [FOLLOWED]?: true };
  if (marked[FOLLOWED]) return;
  marked[FOLLOWED] = true;
  document.addEventListener("click", (event) => {
    const target = event.target as Element | null;
    const pressed = target?.closest(`.${COPY}, .${TOGGLE}`);
    const frame = pressed?.closest(`.${CODE_FRAME}`);
    if (!pressed || !frame) return;
    if (pressed.classList.contains(TOGGLE)) fold(frame, pressed, !frame.classList.contains(FOLDED));
    else copy(frame, pressed);
  });
}

function fold(frame: Element, toggle: Element, folded: boolean): void {
  frame.classList.toggle(FOLDED, folded);
  toggle.setAttribute("aria-expanded", String(!folded));
  toggle.setAttribute("aria-label", folded ? "Show the code" : "Hide the code");
}

function copy(frame: Element, button: Element): void {
  const pre = frame.querySelector(":scope > pre");
  if (!pre) return;
  const document = frame.ownerDocument;
  const select = () => {
    document.getSelection()?.selectAllChildren(pre);
  };
  const clipboard = document.defaultView?.navigator.clipboard;
  if (!clipboard) {
    select();
    return;
  }
  clipboard.writeText(pre.textContent).then(() => {
    button.setAttribute("data-copied", "");
    setTimeout(() => {
      button.removeAttribute("data-copied");
    }, COPIED_FOR);
  }, select);
}

function button(document: Document, className: string, label: string): HTMLButtonElement {
  const element = document.createElement("button");
  element.type = "button";
  element.className = className;
  if (label) element.setAttribute("aria-label", label);
  return element;
}

function setData(element: HTMLElement, key: string, value: string): void {
  if (element.dataset[key] !== value) element.dataset[key] = value;
}
