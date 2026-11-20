import { describe, expect, it } from "vitest";
import { codeBlocks } from "./code-blocks.ts";
import type { ExpandedNode } from "./tree.ts";

const code = (value: string, lang?: string): ExpandedNode => ({ type: "code", value, ...(lang === undefined ? {} : { lang }) });

describe("codeBlocks", () => {
  it("lists each code block's text and language, in slots and fallbacks too, a block without a language as \"\"", () => {
    const nodes: ExpandedNode[] = [
      code("let a = 1;", "ts"),
      code("plain text"),
      {
        type: "componentContainer",
        name: "tabs",
        params: [],
        children: [code("fn main() {}", "rust")],
        slots: [{ type: "componentSlot", name: "footer", children: [code("echo hi", "sh")] }],
      },
      { type: "componentLeaf", name: "chart", params: [], fallback: [code("a,b", "csv")] },
    ];
    expect(codeBlocks(nodes)).toEqual([
      { code: "let a = 1;", language: "ts" },
      { code: "plain text", language: "" },
      { code: "fn main() {}", language: "rust" },
      { code: "echo hi", language: "sh" },
      { code: "a,b", language: "csv" },
    ]);
  });

  it("lists nothing for a document without code", () => {
    expect(codeBlocks([{ type: "paragraph", children: [{ type: "text", value: "No code" }] }])).toEqual([]);
  });
});
