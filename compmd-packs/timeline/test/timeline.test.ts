// The timeline as a host gets it: the pack's files, as a registry gives them, processed with a document that uses it.
// Its expanded tree (spec §12.3) is a `box` with a line on its start side, holding the list the use wrote; and a plain
// Markdown reader shows the list (spec §4.5).
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

/** `text`, a document importing the pack with `milestones.yaml` beside it, processed with the pack's files as its only release. */
function processed(text: string) {
  const manifest = JSON.parse(read("package.json")) as { version: string };
  const request: Request = {
    path: "harbour.comp.md",
    files: { "harbour.comp.md": text, "milestones.yaml": read("milestones.yaml") },
    packs: { "@comp-markdown/timeline": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/timeline@0"\n      names: [timeline]\n---\n\n`;

const TIMELINE = `::: timeline\n- **1897**: the harbour wall is built.\n- **1952**: the quay is rebuilt after the storm.\n::: /timeline\n`;

/** The text of `nodes`, their descendants' values joined. */
const textOf = (nodes: Node[] = []): string => nodes.map((node) => node.value ?? textOf(node.children)).join("");

describe("a timeline", () => {
  it("is a box with a line on its start side, holding the use's list", () => {
    const { expanded, diagnostics } = processed(`${IMPORT}${TIMELINE}`);
    expect(diagnostics).toEqual([]);
    const [timeline] = expanded.children as Node[];
    expect(timeline).toMatchObject({ type: "componentContainer", name: "box" });
    expect(timeline?.params).toEqual([
      { key: "border-inline-start", breakpoint: null, value: "strong" },
      { key: "padding-inline-start", breakpoint: null, value: "m" },
    ]);
    const [list] = timeline?.children ?? [];
    expect(list?.type).toBe("list");
    expect(list?.children?.map((item) => textOf(item.children))).toEqual(["1897: the harbour wall is built.", "1952: the quay is rebuilt after the storm."]);
  });

  it("has the use's line and room in place of its own", () => {
    const [timeline] = processed(`${IMPORT}::: timeline {border-inline-start=subtle padding-inline-start=l}\n- **May 3**: the boats.\n::: /timeline\n`).expanded.children as Node[];
    expect(timeline?.params).toEqual([
      { key: "border-inline-start", breakpoint: null, value: "subtle" },
      { key: "padding-inline-start", breakpoint: null, value: "l" },
    ]);
  });

  it("writes its list from data, one item for each record, in one list", () => {
    const demo = processed(read("demo.comp.md"));
    const examples = (demo.expanded.children as (Node & { document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    const [timeline] = examples[3]?.document?.children ?? [];
    const lists = timeline?.children?.filter((node) => node.type === "list") ?? [];
    expect(lists).toHaveLength(1);
    expect(lists[0]?.children?.map((item) => textOf(item.children))).toEqual([
      "March 2023: the lamp posts are taken down for repair.",
      "October 2023: the first six come back, restored.",
      "April 2024: the last of them is lit again.",
    ]);
  });

  it("reads in a plain Markdown renderer as its list", () => {
    const html = plainHtml(`${IMPORT}${TIMELINE}`);
    expect(html).toContain("<li><strong>1897</strong>: the harbour wall is built.</li>");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both"]);
  });
});
