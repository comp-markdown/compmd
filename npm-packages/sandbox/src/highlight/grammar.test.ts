import { describe, expect, it } from "vitest";
import { splitYaml } from "./grammar.ts";

describe("splitYaml", () => {
  it("splits frontmatter, and a signature before its body", () => {
    expect(splitYaml("---\na: 1\n---\n# T", true)).toEqual(["---\na: 1\n---", "# T"]);
    expect(splitYaml("# T\n\n---\n", true)).toEqual(["", "# T\n\n---\n"]);
    expect(splitYaml("params:\n  a: string\n---\n**{{a}}**", false)).toEqual(["params:\n  a: string\n---", "**{{a}}**"]);
    expect(splitYaml("params:\n  a: string", false)).toEqual(["params:\n  a: string", ""]);
  });
});
