import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { CODE_FRAME, CodeFrame, FOLDED, followCodeFrames, outputTag } from "./code-frame.ts";

let area: HTMLElement;
let pre: HTMLElement;
beforeEach(() => {
  area = document.body.appendChild(document.createElement("main"));
  pre = area.appendChild(document.createElement("pre"));
  pre.textContent = "A --> B\nB --> C";
});
afterEach(() => {
  area.remove();
  Reflect.deleteProperty(navigator, "clipboard");
  document.getSelection()?.removeAllRanges();
  vi.useRealTimers();
});

/** Gives the page's navigator `value` as its clipboard. */
function clipboard(value: Pick<Clipboard, "writeText"> | undefined): void {
  Object.defineProperty(navigator, "clipboard", { value, configurable: true });
}

const bar = () => area.querySelector<HTMLElement>(".compmd-code-bar");
const copy = () => area.querySelector<HTMLButtonElement>(".compmd-code-copy");
const toggle = () => area.querySelector<HTMLButtonElement>(".compmd-code-toggle");

describe("a code block's frame", () => {
  it("takes the code block's place, a bar naming it over it, with no text of its own", () => {
    const frame = new CodeFrame(pre);
    frame.show("mermaid", "A --> B\nB --> C");
    expect(area.firstElementChild).toBe(frame.dom);
    expect(frame.dom.className).toBe(CODE_FRAME);
    expect([...frame.dom.children]).toEqual([bar(), pre]);
    expect(bar()?.dataset).toEqual({ language: "mermaid", lines: "2 lines" });
    expect(copy()?.getAttribute("aria-label")).toBe("Copy the code");
    expect(toggle()).toBeNull();
    expect(frame.dom.textContent).toBe("A --> B\nB --> C");
  });

  it("counts a line, and follows the code as it changes", () => {
    const frame = new CodeFrame(pre);
    frame.show("js", "let a;");
    expect(bar()?.dataset.lines).toBe("1 line");
    frame.show("ts", "let a;\nlet b;\nlet c;");
    expect(bar()?.dataset).toEqual({ language: "ts", lines: "3 lines" });
  });

  it("folds the code with `folds`, and shows it and folds it again on clicks", () => {
    const frame = new CodeFrame(pre, true);
    expect(frame.dom.classList.contains(FOLDED)).toBe(true);
    expect(toggle()?.getAttribute("aria-expanded")).toBe("false");
    expect(toggle()?.getAttribute("aria-label")).toBe("Show the code");
    expect(toggle()?.getAttribute("aria-controls")).toBe(pre.id);
    expect(pre.id).toMatch(/^compmd-code-\d+$/);
    toggle()?.click();
    expect(frame.dom.classList.contains(FOLDED)).toBe(false);
    expect(toggle()?.getAttribute("aria-expanded")).toBe("true");
    expect(toggle()?.getAttribute("aria-label")).toBe("Hide the code");
    toggle()?.click();
    expect(frame.dom.classList.contains(FOLDED)).toBe(true);
  });

  it("keeps an id the code block has", () => {
    pre.id = "mine";
    new CodeFrame(pre, true);
    expect(toggle()?.getAttribute("aria-controls")).toBe("mine");
  });

  it("copies its code, and says so for a moment", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    clipboard({ writeText });
    vi.useFakeTimers();
    new CodeFrame(pre);
    copy()?.click();
    await vi.waitFor(() => {
      expect(copy()?.hasAttribute("data-copied")).toBe(true);
    });
    expect(writeText).toHaveBeenCalledWith("A --> B\nB --> C");
    vi.advanceTimersByTime(1500);
    expect(copy()?.hasAttribute("data-copied")).toBe(false);
  });

  it("selects the code for the reader to copy where the page may not write the clipboard", async () => {
    clipboard({ writeText: () => Promise.reject(new Error("refused")) });
    new CodeFrame(pre);
    copy()?.click();
    await vi.waitFor(() => {
      expect(document.getSelection()?.toString()).toBe("A --> B\nB --> C");
    });
    expect(copy()?.hasAttribute("data-copied")).toBe(false);
  });

  it("selects the code where the page has no clipboard", () => {
    clipboard(undefined);
    new CodeFrame(pre);
    copy()?.click();
    expect(document.getSelection()?.toString()).toBe("A --> B\nB --> C");
  });
});

describe("the frames' listener", () => {
  it("works the buttons of a frame written ahead, as HTML, once per document", () => {
    area.innerHTML = '<div class="compmd-code compmd-folded"><div class="compmd-code-bar"><button class="compmd-code-toggle" aria-expanded="false"></button></div><pre>x</pre></div>';
    followCodeFrames(document);
    followCodeFrames(document);
    toggle()?.click();
    expect(area.firstElementChild?.classList.contains(FOLDED)).toBe(false);
    expect(toggle()?.getAttribute("aria-expanded")).toBe("true");
  });

  it("leaves other clicks alone", () => {
    followCodeFrames(document);
    const other = area.appendChild(document.createElement("button"));
    other.className = "compmd-code-toggle";
    expect(() => {
      other.click();
    }).not.toThrow();
    expect(other.hasAttribute("aria-expanded")).toBe(false);
  });
});

describe("a both block's output tag", () => {
  it("is a box with a word from CSS", () => {
    const tag = outputTag(document);
    expect(tag.className).toBe("compmd-both-tag");
    expect(tag.innerHTML).toBe("<span></span>");
  });
});
