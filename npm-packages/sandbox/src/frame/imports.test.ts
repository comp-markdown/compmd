import { describe, expect, it } from "vitest";
import { withImports } from "./imports.ts";

const urls = { "@kit/d3": "blob:null/d3", "@kit/math": "blob:null/math" };

describe("withImports", () => {
  it("writes static imports, re-exports and import() of a library's name as its URL", async () => {
    const text = [
      'import * as d3 from "@kit/d3";',
      "import { add } from '@kit/math';",
      'export { scale } from "@kit/d3";',
      'export * from "@kit/math";',
      'const lazy = () => import("@kit/d3");',
      "import 'react';",
    ].join("\n");
    expect(await withImports(text, urls)).toBe(
      [
        'import * as d3 from "blob:null/d3";',
        "import { add } from 'blob:null/math';",
        'export { scale } from "blob:null/d3";',
        'export * from "blob:null/math";',
        'const lazy = () => import("blob:null/d3");',
        "import 'react';",
      ].join("\n"),
    );
  });

  it("leaves every other import, import.meta, an import() of an expression and names in strings and comments", async () => {
    const text = 'import { h } from "react";\n// import "@kit/d3"\nconst s = "@kit/d3";\nconst u = import.meta.url;\nconst m = import(name);\nexport const kit = "@kit/d3-extra";\n';
    expect(await withImports(text, urls)).toBe(text);
  });

  it("refuses text that isn't a module", async () => {
    await expect(withImports('import { from "@kit/d3"', urls)).rejects.toThrow();
  });
});
