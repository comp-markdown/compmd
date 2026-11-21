// The card as a host gets it: the pack's files, as a registry gives them, processed with a document that uses it.
// Its expanded tree (spec §12.3) is a `box` with the fence's params under the use's, holding the image, the text and
// the footer in that order; and a plain Markdown reader shows the card's text, image and footer (spec §4.5).
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
  url?: string;
  alt?: string;
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
    packs: { "@comp-markdown/card": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/card@0"\n      names: [card]\n---\n\n`;

const CARD = `::: card {background=subtle border=none}
### The harbour at dusk

A walk along the old quay.

:: #image
![Boats at the quay](harbour.jpg)

:: #footer
[Read the walk](harbour.comp.md)
::: /card
`;

/** The types of `nodes`, and a heading's depth: what a card holds, in order. */
const outline = (nodes: Node[] = []) => nodes.map((node) => (node.type === "heading" ? `heading ${String((node as { depth?: number }).depth)}` : node.type === "paragraph" ? `paragraph: ${node.children?.map((child) => child.type).join(", ") ?? ""}` : node.type));

describe("a card", () => {
  it("is a box with the image, the text, then the footer, its border and padding under the use's", () => {
    const { expanded, diagnostics } = processed(`${IMPORT}${CARD}`);
    expect(diagnostics).toEqual([]);
    const [card] = expanded.children as Node[];
    expect(card?.type).toBe("componentContainer");
    expect(card?.name).toBe("box");
    expect(card?.params).toEqual([
      { key: "padding", breakpoint: null, value: "m" },
      { key: "border-radius", breakpoint: null, value: "m" },
      { key: "background", breakpoint: null, value: "subtle" },
      { key: "border", breakpoint: null, value: "none" },
    ]);
    expect(outline(card?.children)).toEqual(["paragraph: image", "heading 3", "paragraph: text", "paragraph: link"]);
  });

  it("keeps the image the document's own, its path and alt text as written", () => {
    const [card] = processed(`${IMPORT}${CARD}`).expanded.children as Node[];
    const image = card?.children?.[0]?.children?.[0];
    expect(image).toMatchObject({ type: "image", url: "harbour.jpg", alt: "Boats at the quay" });
  });

  it("leaves out an image or a footer it isn't given", () => {
    const [card] = processed(`${IMPORT}::: card\n### Along the canal\n\nTwelve bridges.\n::: /card\n`).expanded.children as Node[];
    expect(outline(card?.children)).toEqual(["heading 3", "paragraph: text"]);
  });

  it("reads in a plain Markdown renderer as its text, then its image, then its footer", () => {
    const html = plainHtml(`${IMPORT}${CARD}`);
    const at = (fragment: string) => html.indexOf(fragment);
    for (const fragment of ["The harbour at dusk", "A walk along the old quay.", 'alt="Boats at the quay"', "Read the walk"]) expect(at(fragment), fragment).toBeGreaterThan(-1);
    expect(at("The harbour at dusk")).toBeLessThan(at('alt="Boats at the quay"'));
    expect(at('alt="Boats at the quay"')).toBeLessThan(at("Read the walk"));
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics: a card each, then three in a grid", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both"]);
    const [grid] = examples[3]?.document?.children ?? [];
    expect(grid?.children?.filter((node) => node.name === "box")).toHaveLength(3);
  });
});
