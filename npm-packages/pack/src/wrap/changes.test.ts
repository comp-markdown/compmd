// What a build says changed: params added, gone and changed, and a param that went still named in the demo or the
// overrides.
import { describe, expect, it } from "vitest";
import { changesOf, declarationsOf, stillNamed } from "./changes.ts";

const fence = (params: string) => `\`\`\`compmd component=qr-code signature\nparams:\n${params}\n\`\`\``;

describe("changesOf", () => {
  it("lists the params added and changed in order, then those gone", () => {
    const before = fence("  value: string\n  size: number\n  level: L | M");
    const after = fence("  value: string\n  level: L | M | H\n  title: string?");
    expect(changesOf(before, after)).toEqual(["`level` was L | M, now L | M | H", "`title` is new: string?", "`size` is gone"]);
    expect(changesOf(after, after)).toEqual([]);
  });

  it("reads the long form's type and default, and nothing from a fence that isn't YAML", () => {
    expect([...declarationsOf(fence("  x:\n    type: string\n    default: a\n    doc: The field"))]).toEqual([["x", 'string = "a"']]);
    expect(declarationsOf(fence("  : [")).size).toBe(0);
    expect(declarationsOf(fence("")).size).toBe(0);
  });
});

describe("stillNamed", () => {
  it("finds a param that went in a use or as an override's key", () => {
    const files = { "demo.comp.md": ":: qr-code[Site] {value=x size=4}\n", "compmd.wrap.json": '{"level": {"doc": "x"}}' };
    expect(stillNamed(["size", "level", "title"], files)).toEqual(["demo.comp.md still names `size`, which the package no longer has", "compmd.wrap.json still names `level`, which the package no longer has"]);
  });
});
