import { describe, expect, it } from "vitest";
import { mdOf, referencesOf } from "./references.ts";

const DOCUMENT = `---
md:
  import:
    - from: ./chart.jsx
      names: [chart]
    - from: "@kit/theme@1"
      names: "*"
  data:
    - from: ./rows.csv
      as: rows
      fallback: ./none.csv
    - from: https://a.org/x.json
      as: remote
    - from: ./{{md.input.region}}.csv
      as: region
  fonts:
    Harbour:
      files:
        - from: ../fonts/harbour.woff2
---

An example's image isn't the pack's: ![](./not-a-definition.png)

\`\`\`compmd component=card
params:
  href: string
---
![A star](./icons/star.svg "Star") [More]({{href}}) [Up](../) [Top](#top) [Site](https://compmd.dev/)
[Spaced](<./a b.svg>) ![](./icons/{{icon}}.svg) [Escape](../../x.svg)

[guide]: ./guide.comp.md#start
\`\`\`
`;

describe("referencesOf", () => {
  it("names what a document's frontmatter and definitions name in its pack, by kind", () => {
    expect(referencesOf("parts/index.comp.md", DOCUMENT)).toEqual({
      imports: ["parts/chart.jsx"],
      data: ["parts/rows.csv", "parts/none.csv"],
      fonts: ["fonts/harbour.woff2"],
      destinations: ["parts/icons/star.svg", "parts/a b.svg", "parts/guide.comp.md"],
      outside: ["../../x.svg"],
      unlisted: ["./{{md.input.region}}.csv", "./icons/{{icon}}.svg"],
    });
  });

  it("finds nothing in a document without frontmatter or definitions", () => {
    expect(referencesOf("index.comp.md", "# A pack\n")).toEqual({ imports: [], data: [], fonts: [], destinations: [], outside: [], unlisted: [] });
  });
});

describe("mdOf", () => {
  it("is the frontmatter's `md` key, or an empty one for a broken or missing one", () => {
    expect(mdOf("---\nmd:\n  version: 1\n---\n")).toEqual({ version: 1 });
    expect(mdOf("---\nmd: [unclosed\n---\n")).toEqual({});
    expect(mdOf("---\nmd: [1]\n---\n")).toEqual({});
    expect(mdOf("# No frontmatter\n")).toEqual({});
  });
});
