import { initSync, process } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { EXAMPLES, textOf } from "./documents.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

describe("EXAMPLES", () => {
  it("opens with the complete example", () => {
    expect(EXAMPLES[0]?.name).toBe("The spec's complete example");
  });

  for (const example of EXAMPLES) {
    it(`${example.name} processes, with only the diagnostics it means to show`, () => {
      expect(textOf(example).length).toBeGreaterThan(0);
      const processed = process(example.request);
      const codes = processed.diagnostics.map((d) => `${d.code} ${String(d.range.start.line)}: ${d.message}`);
      // The release notes import a pack and a component file their folder doesn't hold, as the spec's example does.
      const expected = example.name.startsWith("Release notes") ? 2 : 0;
      expect(codes, codes.join("\n")).toHaveLength(expected);
      expect(processed.expanded.children.length).toBeGreaterThan(1);
    });
  }

  it("gives the dashboard and the field notes the charts with their code", () => {
    const charts = EXAMPLES.filter((example) => example.request.packs?.["@preview/charts"]);
    expect(charts.map((example) => example.name)).toEqual(["A sales dashboard", "Field notes (52 KB, 20 charts)"]);
    const files = Object.keys(charts[0]?.request.packs?.["@preview/charts"]?.[0]?.files ?? {});
    expect(files.sort()).toEqual(["area-chart.jsx", "bar-chart.jsx", "data-table.svelte", "index.comp.md", "package.json", "pie-chart.svelte", "timeline.jsx"]);
  });
});
