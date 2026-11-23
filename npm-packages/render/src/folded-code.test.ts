import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FOLDED, FoldedCode } from "./folded-code.ts";

let box: HTMLElement;
let code: HTMLElement;
beforeEach(() => {
  box = document.body.appendChild(document.createElement("div"));
  code = box.appendChild(document.createElement("pre"));
  code.textContent = "A --> B\nB --> C";
  box.appendChild(document.createElement("div")).textContent = "drawn";
});
afterEach(() => {
  box.remove();
  Reflect.deleteProperty(navigator, "clipboard");
  document.getSelection()?.removeAllRanges();
});

/** Gives the page's navigator `value` as its clipboard. */
function clipboard(value: Pick<Clipboard, "writeText"> | undefined): void {
  Object.defineProperty(navigator, "clipboard", { value, configurable: true });
}

const toggle = () => box.querySelector<HTMLButtonElement>(".compmd-both-toggle");
const copy = () => box.querySelector<HTMLButtonElement>(".compmd-both-copy");

describe("a both box's folded code", () => {
  it("puts a bar naming the code before it and the output's tag after it, folded", () => {
    const folded = new FoldedCode(box, code);
    folded.show("mermaid", "A --> B\nB --> C");
    expect([...box.children].map((child) => child.className || child.nodeName)).toEqual(["compmd-both-bar", "PRE", "compmd-both-tag", "DIV"]);
    expect(box.querySelector(".compmd-both-language")?.textContent).toBe("mermaid");
    expect(box.querySelector(".compmd-both-lines")?.textContent).toBe("2 lines");
    expect(box.querySelector(".compmd-both-tag")?.textContent).toBe("Output");
    expect(folded.folded).toBe(true);
    expect(box.classList.contains(FOLDED)).toBe(true);
    expect(toggle()?.textContent).toBe("Show code");
    expect(toggle()?.getAttribute("aria-expanded")).toBe("false");
    expect(toggle()?.getAttribute("aria-controls")).toBe(code.id);
    expect(code.id).toMatch(/^compmd-code-\d+$/);
  });

  it("shows the code on a click, and folds it again on the next", () => {
    const folded = new FoldedCode(box, code);
    toggle()?.click();
    expect(folded.folded).toBe(false);
    expect(toggle()?.textContent).toBe("Hide code");
    expect(toggle()?.getAttribute("aria-expanded")).toBe("true");
    toggle()?.click();
    expect(folded.folded).toBe(true);
  });

  it("counts a line, and follows the code as it changes", () => {
    const folded = new FoldedCode(box, code);
    folded.show("compmd", "One line.");
    expect(box.querySelector(".compmd-both-lines")?.textContent).toBe("1 line");
    folded.show("compmd", "One.\nTwo.\nThree.");
    expect(box.querySelector(".compmd-both-lines")?.textContent).toBe("3 lines");
  });

  it("keeps an id the code block has", () => {
    code.id = "mine";
    new FoldedCode(box, code);
    expect(toggle()?.getAttribute("aria-controls")).toBe("mine");
  });

  it("copies the code it was shown, and says so", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    clipboard({ writeText });
    vi.useFakeTimers();
    new FoldedCode(box, code).show("mermaid", "A --> B\nB --> C");
    copy()?.click();
    await vi.waitFor(() => {
      expect(copy()?.textContent).toBe("Copied");
    });
    expect(writeText).toHaveBeenCalledWith("A --> B\nB --> C");
    vi.advanceTimersByTime(1500);
    expect(copy()?.textContent).toBe("Copy");
    vi.useRealTimers();
  });

  it("selects the code for the reader to copy where the page may not write the clipboard", async () => {
    clipboard({ writeText: () => Promise.reject(new Error("refused")) });
    new FoldedCode(box, code).show("mermaid", "A --> B\nB --> C");
    copy()?.click();
    await vi.waitFor(() => {
      expect(document.getSelection()?.toString()).toBe("A --> B\nB --> C");
    });
    expect(copy()?.textContent).toBe("Copy");
  });

  it("selects the code where the page has no clipboard", () => {
    clipboard(undefined);
    new FoldedCode(box, code).show("mermaid", "A --> B\nB --> C");
    copy()?.click();
    expect(document.getSelection()?.toString()).toBe("A --> B\nB --> C");
  });
});
