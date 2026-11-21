// The badge as a host gets it: the pack's files, as a registry gives them, processed with a document that uses it.
// Its expanded tree (spec §12.3) is an inline `box`, a `componentInline`, styled by the fence's params under the
// use's, holding the use's fallback; and a plain Markdown reader shows the sentence as written (spec §4.5).
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
  fallback?: Node[];
  value?: string;
}

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** `text`, a document importing the pack with `stops.yaml` beside it, processed with the pack's files as its only release. */
function processed(text: string) {
  const manifest = JSON.parse(read("package.json")) as { version: string };
  const request: Request = {
    path: "walks.comp.md",
    files: { "walks.comp.md": text, "stops.yaml": read("stops.yaml"), "labels.comp.md": read("labels.comp.md") },
    packs: { "@comp-markdown/badge": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/badge@0"\n      names: [badge]\n---\n\n`;

/** The text of `nodes`, their descendants' values joined. */
const textOf = (nodes: Node[] = []): string => nodes.map((node) => node.value ?? textOf(node.children)).join("");

/** The badges in the document's first paragraph. */
function badgesOf(text: string): Node[] {
  const { expanded, diagnostics } = processed(`${IMPORT}${text}\n`);
  expect(diagnostics).toEqual([]);
  const [paragraph] = expanded.children as Node[];
  return (paragraph?.children ?? []).filter((node) => node.type === "componentInline");
}

describe("a badge", () => {
  it("is an inline box in its sentence, its text the use's fallback, subtle by default", () => {
    const [badge] = badgesOf("Faster from version :badge[2.1].");
    expect(badge?.name).toBe("box");
    expect(badge?.params).toEqual([
      { key: "padding-inline", breakpoint: null, value: "s" },
      { key: "border-radius", breakpoint: null, value: "full" },
      { key: "background", breakpoint: null, value: "subtle" },
    ]);
    expect(textOf(badge?.fallback)).toBe("2.1");
  });

  it("takes its background from its tone", () => {
    const badges = badgesOf(":badge[old]{tone=subtle} :badge[stable]{tone=strong} :badge[beta]{tone=accent}");
    expect(badges.map((badge) => badge.params?.find((param) => param.key === "background")?.value)).toEqual(["subtle", "strong", "accent"]);
  });

  it("has the use's params in place of its own: those it keeps, then the use's", () => {
    const [badge] = badgesOf(":badge[closed]{border=strong border-radius=none background=none}");
    expect(badge?.params).toEqual([
      { key: "padding-inline", breakpoint: null, value: "s" },
      { key: "border", breakpoint: null, value: "strong" },
      { key: "border-radius", breakpoint: null, value: "none" },
      { key: "background", breakpoint: null, value: "none" },
    ]);
  });

  it("refuses a tone it hasn't with F009, and is subtle", () => {
    const { expanded, diagnostics } = processed(`${IMPORT}:badge[beta]{tone=loud}\n`);
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toEqual(["F009"]);
    const [paragraph] = expanded.children as Node[];
    expect(paragraph?.children?.[0]?.params?.find((param) => param.key === "background")?.value).toBe("subtle");
  });

  it("reads in a plain Markdown renderer as its sentence, as written", () => {
    expect(plainHtml(`${IMPORT}The new :badge[beta]{tone=accent} search.\n`)).toContain("The new :badge[beta]{tone=accent} search.");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both", "both", "both"]);
    const tones = JSON.stringify(examples[4]?.document?.children).match(/"value":"(accent|subtle)"/g);
    expect(tones).toEqual(['"value":"accent"', '"value":"subtle"', '"value":"accent"']);
  });

  it("is a project's own badges when a file defines them once, each with its look, used with nothing to configure", () => {
    const { expanded, diagnostics } = processed(`---\nmd:\n  import:\n    - from: ./labels.comp.md\n      names: [bug, feature]\n---\n\n:bug[bug] and :feature[new], then :bug[bug] again.\n`);
    expect(diagnostics).toEqual([]);
    const [paragraph] = expanded.children as Node[];
    const badges = (paragraph?.children ?? []).filter((node) => node.type === "componentInline");
    expect(badges.map((badge) => textOf(badge.fallback))).toEqual(["bug", "new", "bug"]);
    expect(badges.map((badge) => badge.params?.find((param) => param.key === "border")?.value)).toEqual(["1px solid #cf222e", "1px solid #1a7f37", "1px solid #cf222e"]);
    expect(badges[0]?.fallback?.[0]?.type).toBe("strong");
  });
});
