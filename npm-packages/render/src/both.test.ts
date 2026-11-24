import { describe, expect, it } from "vitest";
import { plainCode, showBoth } from "./both.ts";
import type { View } from "./render.ts";
import type { ExpandedNode } from "./tree.ts";

describe("a block that shows its code and its output", () => {
  it("is its code alone, without what renders it", () => {
    const rendered: ExpandedNode = { type: "code", lang: "mermaid", meta: "{render=both}", value: "A --> B", render: "both", alt: "A, then B", component: { name: "mermaid", params: [] } };
    expect(plainCode(rendered)).toEqual({ type: "code", lang: "mermaid", meta: "{render=both}", value: "A --> B" });
  });

  it("puts the code, then the output's tag, in its box, marked as render=both's", () => {
    const box = document.createElement("div");
    const code = document.createElement("pre");
    showBoth(box, { dom: code } as unknown as View);
    expect(box.classList.contains("compmd-both")).toBe(true);
    expect([...box.children].map((child) => child.className || child.tagName)).toEqual(["PRE", "compmd-both-tag"]);
  });
});
