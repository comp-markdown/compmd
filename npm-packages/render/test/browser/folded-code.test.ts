// A `render=both` block's code, folded on screen (spec §7.7): hidden under its bar until the reader shows it, the output
// then tagged in its corner; on paper, the code without the bar or the tag.
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

/** A Mermaid block with `render=both`, rendered on a page; its box. */
function both(): HTMLElement {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 480px; font: 16px/1.5 sans-serif";
  const block: ExpandedNode = { type: "code", lang: "mermaid", meta: null, value: "A --> B", render: "both", alt: "A, then B", component: { name: "mermaid", params: [{ key: "code", breakpoint: null, value: "A --> B" }] } };
  createPage(root, { components: { mermaid } }).render({ type: "root", children: [block] });
  return root.querySelector<HTMLElement>(".compmd-both") as HTMLElement;
}

const display = (element: Element | null) => (element ? getComputedStyle(element).display : "missing");

/** The base rules for paper: the text of each print rule in the page's sheets. */
function printRules(): string {
  const sheets = [...document.adoptedStyleSheets, ...document.styleSheets];
  return sheets
    .flatMap((sheet) => [...sheet.cssRules])
    .filter((rule): rule is CSSMediaRule => rule instanceof CSSMediaRule && rule.media.mediaText === "print")
    .map((rule) => rule.cssText)
    .join("\n");
}

describe("a both block's folded code", () => {
  it("hides the code and the tag on screen until the reader shows them", () => {
    const box = both();
    const pre = box.querySelector("pre");
    const tag = box.querySelector(".compmd-both-tag");
    expect(display(pre)).toBe("none");
    expect(display(tag)).toBe("none");
    expect(display(box.querySelector(".compmd-both-copy"))).toBe("none");
    expect(display(box.querySelector(".compmd-both-bar"))).toBe("flex");
    const bar = box.querySelector(".compmd-both-bar") as HTMLElement;
    expect((box.lastElementChild as HTMLElement).getBoundingClientRect().top).toBeGreaterThan(bar.getBoundingClientRect().bottom);
    box.querySelector<HTMLButtonElement>(".compmd-both-toggle")?.click();
    expect(display(pre)).toBe("block");
    expect(display(tag)).toBe("flex");
    expect(display(box.querySelector(".compmd-both-copy"))).not.toBe("none");
  });

  it("tags the output in its top corner, taking no room of its own", () => {
    const box = both();
    box.querySelector<HTMLButtonElement>(".compmd-both-toggle")?.click();
    const tag = box.querySelector(".compmd-both-tag") as HTMLElement;
    const label = tag.firstElementChild as HTMLElement;
    const output = box.lastElementChild as HTMLElement;
    expect(tag.getBoundingClientRect().height).toBe(0);
    expect(Math.round(label.getBoundingClientRect().right)).toBe(Math.round(box.getBoundingClientRect().right));
    expect(Math.round(label.getBoundingClientRect().top)).toBe(Math.round(output.getBoundingClientRect().top));
    expect(label.getBoundingClientRect().height).toBeGreaterThan(0);
  });

  it("prints the code, without the bar or the tag", () => {
    both();
    const rules = printRules();
    expect(rules).toContain(".compmd-both-bar");
    expect(rules).toContain(".compmd-both-tag");
    expect(rules).not.toContain("compmd-folded");
  });
});
