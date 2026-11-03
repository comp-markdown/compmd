import type { Components } from "@compmd/processor";
import { describe, expect, it } from "vitest";
import { kindLabel, listsOf, paramText, slotsText, sourceText, typeText, usesText } from "./components.ts";

const at = (line: number) => ({ start: { line, column: 4 }, end: { line, column: 8 } });

describe("the Components tab's lists", () => {
  const components: Components = {
    groups: [
      {
        provided: false,
        components: [
          { name: "note", kind: "compmd", defined: { start: { line: 3, column: 1 }, end: { line: 6, column: 4 } }, uses: [at(9)] },
          { name: "spare", kind: "compmd", uses: [] },
        ],
      },
      { file: "kit.comp.md", provided: false, components: [{ name: "chip", kind: "compmd", uses: [] }] },
      {
        file: "@compmd/core@1.0.0/index.comp.md",
        provided: true,
        components: [
          { name: "box", kind: "processor", uses: [at(2)] },
          { name: "each", kind: "processor", uses: [] },
        ],
      },
    ],
    unknown: [{ name: "gone", uses: [at(5)] }],
  };

  it("separates what the document defines from what it uses, and what it could use", () => {
    const { defined, used, available } = listsOf(components);
    expect(defined.map((c) => c.name)).toEqual(["note", "spare"]);
    expect(defined[0]?.defined, "where its definition is, for its link").toEqual({ start: { line: 3, column: 1 }, end: { line: 6, column: 4 } });
    expect(used).toEqual([
      { name: "box", kind: "processor", uses: [at(2)], source: "@compmd/core@1.0.0/index.comp.md" },
      { name: "gone", unknown: true, uses: [at(5)] },
      { name: "note", kind: "compmd", defined: { start: { line: 3, column: 1 }, end: { line: 6, column: 4 } }, uses: [at(9)], source: undefined },
    ]);
    expect(available.map((c) => [c.name, c.source])).toEqual([
      ["chip", "kit.comp.md"],
      ["each", "@compmd/core@1.0.0/index.comp.md"],
    ]);
  });

  it("names where a component comes from", () => {
    expect([undefined, "kit.comp.md", "@compmd/core@1.0.0/index.comp.md", "@acme/charts@1.4.2/chart.jsx"].map(sourceText)).toEqual([
      "this document",
      "kit.comp.md",
      "@compmd/core@1.0.0",
      "@acme/charts@1.4.2/chart.jsx",
    ]);
  });
});

describe("the Components tab's words", () => {
  it("names each kind", () => {
    expect(["compmd", "processor", "jsx", "svelte", "vue"].map(kindLabel)).toEqual(["CompMD", "Built in", "JSX", "Svelte", "vue"]);
  });

  it("writes params as a signature does", () => {
    expect(typeText({ enum: ["draft", "final"] })).toBe("draft | final");
    expect(typeText({ input: "date" })).toBe("input<date>");
    expect(paramText({ name: "tone", type: "string", optional: true, default: null, doc: null })).toBe("tone: string?");
    expect(paramText({ name: "n", type: "number", optional: false, default: null, doc: null })).toBe("n: number");
  });

  it("lists slots, and none for a component without a body", () => {
    const listed = (slots: (string | { name: string; each: string; as: string })[] | null) => ({
      name: "c",
      kind: "compmd",
      uses: [],
      signature: { params: [], slots, headings: "relative" as const },
    });
    expect(slotsText(listed(["default", { name: "row", each: "rows", as: "r" }]))).toBe("default, row (each rows)");
    expect(slotsText(listed(null))).toBeUndefined();
    expect(slotsText({ name: "c", kind: "jsx", uses: [] })).toBeUndefined();
  });

  it("counts uses", () => {
    expect([0, 1, 3].map(usesText)).toEqual(["not used", "1 use", "3 uses"]);
  });
});
