// The sticky note as a host gets it: the pack's files, as a registry gives them, processed with a document that uses
// it. Its expanded tree (spec §12.3) is a `box`, tinted, tilted and with a shadow, holding what the use wrote, with the
// use's position after the fence's params; and a plain Markdown reader shows the note's text (spec §4.5).
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
    path: "walks.comp.md",
    files: { "walks.comp.md": text },
    packs: { "@comp-markdown/sticky-note": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/sticky-note@0"\n      names: [sticky-note]\n---\n\n`;

const NOTE = `::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=4rem}\nBring the tide table: the quay floods at high water.\n::: /sticky-note\n`;

const FENCE = [
  { key: "padding", breakpoint: null, value: "m" },
  { key: "background", breakpoint: null, value: "accent" },
  { key: "shadow", breakpoint: null, value: "m" },
  { key: "rotate", breakpoint: null, value: "-2deg" },
  { key: "width", breakpoint: null, value: "14rem" },
  { key: "max-width", breakpoint: null, value: "100%" },
];

/** The text of `nodes`, their descendants' values joined. */
const textOf = (nodes: Node[] = []): string => nodes.map((node) => node.value ?? textOf(node.children)).join("");

describe("a sticky note", () => {
  it("is a tinted, tilted box with a shadow, holding what the use wrote", () => {
    const { expanded, diagnostics } = processed(`${IMPORT}::: sticky-note\nClosed on Mondays.\n::: /sticky-note\n`);
    expect(diagnostics).toEqual([]);
    const [note] = expanded.children as Node[];
    expect(note).toMatchObject({ type: "componentContainer", name: "box" });
    expect(note?.params).toEqual(FENCE);
    expect(textOf(note?.children)).toBe("Closed on Mondays.");
  });

  it("is pinned by the use's insets, from their breakpoint up", () => {
    const [note] = processed(`${IMPORT}${NOTE}`).expanded.children as Node[];
    expect(note?.params).toEqual([
      ...FENCE,
      { key: "inset-inline-end", breakpoint: "sm", value: "1rem" },
      { key: "inset-block-start", breakpoint: "sm", value: "4rem" },
    ]);
  });

  it("has the use's tilt and tint in place of its own", () => {
    const [note] = processed(`${IMPORT}::: sticky-note {rotate=3deg background=subtle}\nClosed.\n::: /sticky-note\n`).expanded.children as Node[];
    expect(note?.params?.filter((param) => param.key === "rotate" || param.key === "background")).toEqual([
      { key: "rotate", breakpoint: null, value: "3deg" },
      { key: "background", breakpoint: null, value: "subtle" },
    ]);
  });

  it("refuses a tilt beyond 15 degrees with F009", () => {
    const { diagnostics } = processed(`${IMPORT}::: sticky-note {rotate=40deg}\nClosed.\n::: /sticky-note\n`);
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toEqual(["F009"]);
  });

  it("reads in a plain Markdown renderer as its text", () => {
    expect(plainHtml(`${IMPORT}${NOTE}`)).toContain("Bring the tide table: the quay floods at high water.");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both"]);
  });
});
