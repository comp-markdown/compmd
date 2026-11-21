// The columns as a host gets them: the pack's files, as a registry gives them, processed with a document that uses
// them. Its expanded tree (spec §12.3) is a grid `box` of one box for each column given, its tracks from `split` and
// how many there are, from `sm` up; and a plain Markdown reader shows the columns in order (spec §4.5).
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { initSync, plainHtml, process as processDocument, type Request } from "@compmd/processor";
import { beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const read = (file: string) => readFileSync(join(ROOT, file), "utf8");

interface Node {
  type: string;
  name?: string;
  params?: { key: string; breakpoint: string | null; value: unknown }[];
  children?: Node[];
  value?: string;
}

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** `text`, a document importing the pack, processed with the pack's files as its only release. */
function processed(text: string) {
  const manifest = JSON.parse(read("package.json")) as { version: string };
  const request: Request = {
    path: "recipe.comp.md",
    files: { "recipe.comp.md": text },
    packs: { "@comp-markdown/columns": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/columns@0"\n      names: [columns]\n---\n\n`;

const TWO = (params = "") => `${IMPORT}::: columns${params}\nThe first.\n\n:: #end\nThe last.\n::: /columns\n`;
const THREE = (params = "") => `${IMPORT}::: columns${params}\nThe first.\n\n:: #middle\nThe middle.\n\n:: #end\nThe last.\n::: /columns\n`;

/** The text of `nodes`, their descendants' values joined. */
const textOf = (nodes: Node[] = []): string => nodes.map((node) => node.value ?? textOf(node.children)).join("");

/** The columns' grid and its column boxes. */
function columnsOf(text: string) {
  const { expanded, diagnostics } = processed(text);
  expect(diagnostics).toEqual([]);
  const [grid] = expanded.children as Node[];
  return { grid, columns: (grid?.children ?? []).filter((node) => node.name === "box") };
}

const tracks = (grid: Node | undefined) => grid?.params?.find((param) => param.key === "grid-template-columns");

describe("columns", () => {
  it("are a grid of a box for each column, its tracks from sm up, with a gap", () => {
    const { grid, columns } = columnsOf(TWO());
    expect(grid?.params).toEqual([
      { key: "display", breakpoint: null, value: "grid" },
      { key: "gap", breakpoint: null, value: "m" },
      { key: "grid-template-columns", breakpoint: "sm", value: "1fr 1fr" },
    ]);
    expect(columns.map((column) => textOf(column.children))).toEqual(["The first.", "The last."]);
  });

  it("have three boxes and three tracks with a middle", () => {
    const { grid, columns } = columnsOf(THREE());
    expect(columns.map((column) => textOf(column.children))).toEqual(["The first.", "The middle.", "The last."]);
    expect(tracks(grid)?.value).toBe("1fr 1fr 1fr");
  });

  it("take their widths from split, for two and for three", () => {
    const cases: [string, string, string][] = [
      ["wide-start", "2fr 1fr", "2fr 1fr 1fr"],
      ["wide-end", "1fr 2fr", "1fr 1fr 2fr"],
      ["wide-middle", "1fr 2fr 1fr", "1fr 2fr 1fr"],
    ];
    for (const [split, two, three] of cases) {
      expect(tracks(columnsOf(TWO(` {split=${split}}`)).grid)?.value, split).toBe(two);
      expect(tracks(columnsOf(THREE(` {split=${split}}`)).grid)?.value, split).toBe(three);
    }
  });

  it("refuse a split they haven't with F009, and are even", () => {
    const { expanded, diagnostics } = processed(TWO(" {split=golden}"));
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toEqual(["F009"]);
    expect(tracks((expanded.children as Node[])[0])?.value).toBe("1fr 1fr");
  });

  it("have the use's gap in place of their own", () => {
    const { grid } = columnsOf(TWO(" {gap=l}"));
    expect(grid?.params?.find((param) => param.key === "gap")?.value).toBe("l");
  });

  it("read in a plain Markdown renderer as their columns, in order", () => {
    const html = plainHtml(THREE());
    const at = (fragment: string) => html.indexOf(fragment);
    expect(at("The first.")).toBeGreaterThan(-1);
    expect(at("The first.")).toBeLessThan(at("The middle."));
    expect(at("The middle.")).toBeLessThan(at("The last."));
  });

  it("draw each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both", "both"]);
  });
});
