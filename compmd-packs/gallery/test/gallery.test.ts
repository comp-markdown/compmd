// The gallery as a host gets it: its pack's and the card pack's files, as a registry gives them, processed with a
// document that uses it over a list in its data. Its expanded tree (spec §12.3) is a grid of one card for each item,
// each with the item's image, title and description; the images' paths are the document's (spec §8.6); and a plain
// Markdown reader shows the gallery's sentence (spec §4.5).
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { initSync, plainHtml, process as processDocument, type Request } from "@compmd/processor";
import { beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const CARD = join(ROOT, "../card");
const read = (dir: string, file: string) => readFileSync(join(dir, file), "utf8");

interface Node {
  type: string;
  name?: string;
  params?: { key: string; breakpoint: string | null; value: unknown }[];
  children?: Node[];
  value?: string;
  url?: string;
  alt?: string;
  depth?: number;
}

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** A release of the pack in `dir`, its files as a registry gives them. */
const releaseOf = (dir: string) => [{ version: (JSON.parse(read(dir, "package.json")) as { version: string }).version, files: { "package.json": read(dir, "package.json"), "index.comp.md": read(dir, "index.comp.md") } }];

/** `text`, a document at `path` with `walks.yaml` beside it, processed with the gallery and the card as their only releases. */
function processed(text: string, path = "walks.comp.md") {
  const folder = path.includes("/") ? path.slice(0, path.lastIndexOf("/") + 1) : "";
  const request: Request = {
    path,
    files: { [path]: text, [`${folder}walks.yaml`]: read(ROOT, "walks.yaml"), [`${folder}photos.yaml`]: read(ROOT, "photos.yaml"), [`${folder}shapes.yaml`]: read(ROOT, "shapes.yaml") },
    packs: { "@comp-markdown/gallery": releaseOf(ROOT), "@comp-markdown/card": releaseOf(CARD) },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const USE = `---\nmd:\n  import:\n    - from: "@comp-markdown/gallery@0"\n      names: [gallery]\n  data:\n    - from: ./walks.yaml\n      as: walks\n---\n\n## Walks\n\n:: gallery[Three walks around the harbour] {items=(md.data.walks)}\n`;

/** The gallery's grid, from the document's expanded tree: the box the gallery is, then the grid's box in it. */
function gridOf(children: Node[]): Node | undefined {
  return children.find((node) => node.type === "componentContainer")?.children?.[0];
}

/** What a card holds, in order: its image's path, its heading's level and text, its description. */
function cardOf(card: Node): (string | undefined)[] {
  return (card.children ?? []).map((node) => {
    if (node.type === "heading") return `heading ${String(node.depth)}: ${node.children?.map((child) => child.value).join("") ?? ""}`;
    const image = node.children?.find((child) => child.type === "image");
    return image ? `image ${image.url ?? ""} (${image.alt ?? ""})` : `text: ${node.children?.map((child) => child.value).join("") ?? ""}`;
  });
}

describe("a gallery", () => {
  it("is a grid of one card for each item, with its image, its title one level below the heading above, and its description", () => {
    const { expanded, diagnostics } = processed(USE);
    expect(diagnostics).toEqual([]);
    const grid = gridOf(expanded.children as Node[]);
    expect(grid?.params).toEqual(expect.arrayContaining([{ key: "grid-template-columns", breakpoint: null, value: "repeat(auto-fill, minmax(14rem, 1fr))" }]));
    const cards = grid?.children?.filter((node) => node.name === "box") ?? [];
    expect(cards.map(cardOf)).toEqual([
      ["image images/harbour.svg (Boats at the quay, the lamps just lit)", "heading 3: The harbour at dusk", "text: A walk along the old quay, an hour before the boats come in."],
      ["image images/orchard.svg (Rows of apple trees on a slope)", "heading 3: The old orchard", "text: Apples in October, and a bench at the top of the hill."],
      ["image images/canal.svg (A canal under a low stone bridge)", "heading 3: Along the canal"],
    ]);
  });

  it("keeps the images the document's own, seen from where the document is", () => {
    const { expanded, diagnostics } = processed(USE, "posts/walks.comp.md");
    expect(diagnostics).toEqual([]);
    const [first] = gridOf(expanded.children as Node[])?.children?.filter((node) => node.name === "box") ?? [];
    expect(first && cardOf(first)[0]).toBe("image images/harbour.svg (Boats at the quay, the lamps just lit)");
    expect(JSON.stringify(expanded)).not.toContain('"pack":true');
  });

  it("reads in a plain Markdown renderer as its sentence", () => {
    expect(plainHtml(USE)).toContain("Three walks around the harbour");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read(ROOT, "demo.comp.md"), "demo.comp.md");
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both", "both", "both", "both"]);
    const cards = examples.map((example) => gridOf(example.document?.children ?? [])?.children?.filter((node) => node.name === "box").length);
    expect(cards).toEqual([3, 2, 12, 3, 3, 3, 3]);
  });
});

/** The grid's params for the use `params` gives, as `key` or `breakpoint:key` and its value. */
function gridParams(params: string): string[] {
  const { expanded, diagnostics } = processed(USE.replace("{items=(md.data.walks)}", `{items=(md.data.walks) ${params}}`));
  expect(diagnostics).toEqual([]);
  return (gridOf(expanded.children as Node[])?.params ?? []).map(({ key, breakpoint, value }) => `${breakpoint ? `${breakpoint}:` : ""}${key}=${String(value)}`);
}

describe("a gallery's grid", () => {
  it("fits as many cards as their narrowest width allows, `min`, with `gap` between them", () => {
    expect(gridParams("min=10rem gap=s")).toEqual(expect.arrayContaining(["grid-template-columns=repeat(auto-fill, minmax(10rem, 1fr))", "gap=s"]));
    expect(gridParams("")).toEqual(expect.arrayContaining(["gap=m"]));
  });

  it("makes every row as tall as the tallest", () => {
    expect(gridParams("")).toEqual(expect.arrayContaining(["grid-auto-rows=1fr"]));
  });

  it("crops every image to `ratio`, filled as `image-fit` says and placed as `image-position` says, and leaves each its own shape without it", () => {
    const { expanded, diagnostics } = processed(USE.replace("{items=(md.data.walks)}", "{items=(md.data.walks) ratio=4/3 image-position=start}"));
    expect(diagnostics).toEqual([]);
    const cards = gridOf(expanded.children as Node[])?.children?.filter((node) => node.name === "box") ?? [];
    const [shape] = cards.map((card) => card.children?.[0]);
    expect(shape?.name).toBe("box");
    expect(shape?.params?.map(({ key, value }) => `${key}=${String(value)}`)).toEqual(["aspect-ratio=4/3", "object-fit=cover", "object-position=start"]);
    expect(shape && cardOf({ type: "box", children: shape.children ?? [] })).toEqual(["image images/harbour.svg (Boats at the quay, the lamps just lit)"]);
    const [plain] = (gridOf(processed(USE).expanded.children as Node[])?.children ?? []).map((card) => card.children?.[0]);
    expect(plain?.type).toBe("paragraph");
  });

  it("puts exactly `columns` side by side, in place of `min`, at each breakpoint it is given", () => {
    expect(gridParams("columns=2")).toEqual(expect.arrayContaining(["grid-template-columns=repeat(2, minmax(0, 1fr))"]));
    const responsive = gridParams("columns=1 md:columns=3");
    expect(responsive).toEqual(expect.arrayContaining(["grid-template-columns=repeat(1, minmax(0, 1fr))", "md:grid-template-columns=repeat(3, minmax(0, 1fr))"]));
    expect(responsive.filter((param) => param.includes("auto-fill"))).toEqual([]);
  });
});
