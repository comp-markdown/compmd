// A code block's frame (`code-frame.ts`): its bar's words from CSS, and a `render=both` block's code folded on screen
// until the reader shows it, the output then tagged in its corner; on paper, the code without the bar or the tag.
import { afterEach, describe, expect, it } from "vitest";
import type { Mount } from "../../src/contract.ts";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const mermaid: Mount = (target) => {
  target.textContent = "drawn";
};

/** `block` rendered on a page. */
function rendered(block: ExpandedNode): void {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 480px; font: 16px/1.5 sans-serif";
  createPage(root, { components: { mermaid } }).render({ type: "root", children: [block] });
}

/** A Mermaid block with `render=both`, rendered; its box. */
function both(): HTMLElement {
  rendered({ type: "code", lang: "mermaid", meta: null, value: "A --> B", render: "both", alt: "A, then B", component: { name: "mermaid", params: [{ key: "code", breakpoint: null, value: "A --> B" }] } });
  return root.querySelector<HTMLElement>(".compmd-both") as HTMLElement;
}

const display = (element: Element | null) => (element ? getComputedStyle(element).display : "missing");
const word = (element: Element | null, pseudo = "::before") => (element ? getComputedStyle(element, pseudo).content : "missing");

/** The text of each print rule in the page's sheets. */
function printRules(): string {
  return [...document.adoptedStyleSheets, ...document.styleSheets]
    .flatMap((sheet) => [...sheet.cssRules])
    .filter((rule): rule is CSSMediaRule => rule instanceof CSSMediaRule && rule.media.mediaText === "print")
    .map((rule) => rule.cssText)
    .join("\n");
}

describe("a code block's frame", () => {
  it("names the code and its lines, with Copy, all words from CSS", () => {
    rendered({ type: "code", lang: "js", meta: null, value: "let a;\nlet b;" });
    const bar = root.querySelector(".compmd-code-bar");
    expect(display(bar)).toBe("flex");
    expect(word(bar)).toBe('"js"');
    expect(word(bar, "::after")).toBe('"2 lines"');
    expect(word(root.querySelector(".compmd-code-copy"))).toBe('"Copy"');
    expect(root.querySelector(".compmd-code-toggle")).toBeNull();
    expect(display(root.querySelector("pre"))).toBe("block");
  });
});

describe("a both block's frame", () => {
  it("hides the code, Copy and the tag on screen until the reader shows them", () => {
    const box = both();
    const pre = box.querySelector("pre");
    const tag = box.querySelector(".compmd-both-tag");
    const toggle = box.querySelector<HTMLButtonElement>(".compmd-code-toggle");
    expect(display(pre)).toBe("none");
    expect(display(tag)).toBe("none");
    expect(display(box.querySelector(".compmd-code-copy"))).toBe("none");
    expect(word(toggle)).toBe('"Show code"');
    const bar = box.querySelector(".compmd-code-bar") as HTMLElement;
    expect((box.lastElementChild as HTMLElement).getBoundingClientRect().top).toBeGreaterThan(bar.getBoundingClientRect().bottom);
    toggle?.click();
    expect(display(pre)).toBe("block");
    expect(display(tag)).toBe("flex");
    expect(display(box.querySelector(".compmd-code-copy"))).not.toBe("none");
    expect(word(toggle)).toBe('"Hide code"');
  });

  it("tags the output in its top corner, taking no room of its own", () => {
    const box = both();
    box.querySelector<HTMLButtonElement>(".compmd-code-toggle")?.click();
    const tag = box.querySelector(".compmd-both-tag") as HTMLElement;
    const label = tag.firstElementChild as HTMLElement;
    const output = box.lastElementChild as HTMLElement;
    expect(tag.getBoundingClientRect().height).toBe(0);
    expect(word(label)).toBe('"Output"');
    expect(Math.round(label.getBoundingClientRect().right)).toBe(Math.round(box.getBoundingClientRect().right));
    expect(Math.round(label.getBoundingClientRect().top)).toBe(Math.round(output.getBoundingClientRect().top));
    expect(label.getBoundingClientRect().height).toBeGreaterThan(0);
  });

  it("prints the code, without the bar or the tag", () => {
    both();
    const rules = printRules();
    expect(rules).toContain(".compmd-code-bar");
    expect(rules).toContain(".compmd-both-tag");
    expect(rules).not.toContain("compmd-folded");
  });
});
