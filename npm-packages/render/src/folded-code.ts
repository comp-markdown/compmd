// A `render=both` block's code, folded on screen until the reader asks for it (spec §7.7): a bar over the output names
// the code (its language, its line count) with a button that shows it and one that copies it, and a tag names the
// output while the code shows. On paper the code prints, then the output, and the bar and the tag don't.
import { lineCount } from "./code-lines.ts";

/** The class of a `both` box whose code is folded. */
export const FOLDED = "compmd-folded";

let made = 0;

/**
 * The bar and the tag of the `both` box `box`, around its code block `code`, which it folds.
 *
 * @example
 * const folded = new FoldedCode(box, pre);
 * folded.show("mermaid", "A --> B"); // the bar reads "mermaid · 1 line"
 */
export class FoldedCode {
  readonly #box: HTMLElement;
  readonly #code: HTMLElement;
  readonly #language: HTMLElement;
  readonly #lines: HTMLElement;
  readonly #copy: HTMLButtonElement;
  readonly #toggle: HTMLButtonElement;
  readonly #tag: HTMLElement;
  #text = "";

  /** Puts the bar before `code` in `box` and the tag after it, and folds the code. */
  constructor(box: HTMLElement, code: HTMLElement) {
    const document = box.ownerDocument;
    this.#box = box;
    this.#code = code;
    code.id ||= `compmd-code-${String(++made)}`;
    const bar = document.createElement("div");
    bar.className = "compmd-both-bar";
    this.#language = bar.appendChild(document.createElement("span"));
    this.#language.className = "compmd-both-language";
    this.#lines = bar.appendChild(document.createElement("span"));
    this.#lines.className = "compmd-both-lines";
    this.#copy = bar.appendChild(button(document, "compmd-both-copy", "Copy"));
    this.#copy.addEventListener("click", () => {
      this.#copyCode();
    });
    this.#toggle = bar.appendChild(button(document, "compmd-both-toggle", ""));
    this.#toggle.setAttribute("aria-controls", code.id);
    this.#toggle.addEventListener("click", () => {
      this.#fold(!this.folded);
    });
    this.#tag = document.createElement("div");
    this.#tag.className = "compmd-both-tag";
    this.#tag.appendChild(document.createElement("span")).textContent = "Output";
    code.before(bar);
    code.after(this.#tag);
    this.#fold(true);
  }

  /** Whether the code is folded. */
  get folded(): boolean {
    return this.#box.classList.contains(FOLDED);
  }

  /** Names the code, in `language`, with `text`, which Copy copies. */
  show(language: string, text: string): void {
    this.#text = text;
    if (this.#language.textContent !== language) this.#language.textContent = language;
    const lines = lineCount(text);
    const count = lines === 1 ? "1 line" : `${String(lines)} lines`;
    if (this.#lines.textContent !== count) this.#lines.textContent = count;
  }

  #fold(folded: boolean): void {
    this.#box.classList.toggle(FOLDED, folded);
    this.#toggle.setAttribute("aria-expanded", String(!folded));
    this.#toggle.textContent = folded ? "Show code" : "Hide code";
  }

  /** Copies the code; where the page may not write the clipboard, selects it, for the reader to copy. */
  #copyCode(): void {
    const clipboard = this.#box.ownerDocument.defaultView?.navigator.clipboard;
    const select = () => {
      this.#box.ownerDocument.getSelection()?.selectAllChildren(this.#code);
    };
    if (!clipboard) {
      select();
      return;
    }
    clipboard.writeText(this.#text).then(() => {
      this.#copy.textContent = "Copied";
      setTimeout(() => {
        this.#copy.textContent = "Copy";
      }, 1500);
    }, select);
  }
}

function button(document: Document, className: string, text: string): HTMLButtonElement {
  const element = document.createElement("button");
  element.type = "button";
  element.className = className;
  element.textContent = text;
  return element;
}
