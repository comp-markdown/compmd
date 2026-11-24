// The figure as a host gets it: the pack's files, as a registry gives them, processed with a document that uses it.
// Its expanded tree (spec §12.3) is a box with the image, whose alt is the caption's text, then the caption; the
// image's path is the document's (spec §8.6); it prints as it shows, plain Markdown with no component left to draw
// (spec §14); and a plain Markdown reader shows the caption (spec §4.5).
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
  pack?: boolean;
}

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** `text`, a document at `path` importing the pack, processed with the pack's files as its only release. */
function processed(text: string, path = "walk.comp.md") {
  const manifest = JSON.parse(read("package.json")) as { version: string };
  const request: Request = {
    path,
    files: { [path]: text },
    packs: { "@comp-markdown/figure": [{ version: manifest.version, files: { "package.json": read("package.json"), "index.comp.md": read("index.comp.md") } }] },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/figure@0"\n      names: [figure]\n---\n\n`;
const USE = `${IMPORT}:: figure[Boats at the quay, the lamps just lit.] {src="images/harbour.jpg" width=20rem}\n`;

/** Every node of `nodes`, depth first. */
const all = (nodes: Node[] = []): Node[] => nodes.flatMap((node) => [node, ...all(node.children)]);

describe("a figure", () => {
  it("is a box with the image, whose alt is the caption, then the caption, its spacing under the use's params", () => {
    const { expanded, diagnostics } = processed(USE);
    expect(diagnostics).toEqual([]);
    const [figure] = expanded.children as Node[];
    expect(figure).toMatchObject({ type: "componentContainer", name: "box" });
    expect(figure?.params).toEqual([
      { key: "padding-block", breakpoint: null, value: "s" },
      { key: "width", breakpoint: null, value: "20rem" },
    ]);
    const [image, caption] = figure?.children ?? [];
    expect(image?.children?.[0]).toMatchObject({ type: "image", url: "images/harbour.jpg", alt: "Boats at the quay, the lamps just lit." });
    expect(caption).toMatchObject({ type: "paragraph", children: [{ type: "text", value: "Boats at the quay, the lamps just lit." }] });
  });

  it("keeps the image the document's own, seen from where the document is, and never the pack's", () => {
    const { expanded } = processed(USE, "posts/walk.comp.md");
    const image = all(expanded.children as Node[]).find((node) => node.type === "image");
    expect(image).toMatchObject({ url: "images/harbour.jpg" });
    expect(image?.pack).toBeUndefined();
  });

  it("prints as it shows: plain Markdown, with no component left for a host to draw", () => {
    const nodes = all(processed(USE).expanded.children);
    expect(nodes.map((node) => node.type)).not.toContain("componentLeaf");
    expect(nodes.filter((node) => node.type.startsWith("component")).map((node) => node.name)).toEqual(["box"]);
  });

  it("reads in a plain Markdown renderer as its caption", () => {
    expect(plainHtml(USE)).toContain("Boats at the quay, the lamps just lit.");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"), "demo.comp.md");
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both"]);
    expect(examples.map((example) => all(example.document?.children).filter((node) => node.type === "image").length)).toEqual([1, 1, 1]);
  });
});
