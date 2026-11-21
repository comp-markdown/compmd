// The link card as a host gets it: its pack's and the card pack's files, as a registry gives them, processed with a
// document that uses it. Its expanded tree (spec §12.3) is a box with `stretch-link` holding a card, whose heading is
// the link, its text the use's fallback and its destination the document's (spec §8.6); and a plain Markdown reader
// shows the use's line, its title in it (spec §4.5).
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
  depth?: number;
}

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** A release of the pack in `dir`, its files as a registry gives them. */
const releaseOf = (dir: string) => [{ version: (JSON.parse(read(dir, "package.json")) as { version: string }).version, files: { "package.json": read(dir, "package.json"), "index.comp.md": read(dir, "index.comp.md") } }];

/** `text`, a document at `path`, processed with the link card and the card as their only releases. */
function processed(text: string, path = "walks.comp.md") {
  const request: Request = {
    path,
    files: { [path]: text },
    packs: { "@comp-markdown/link-card": releaseOf(ROOT), "@comp-markdown/card": releaseOf(CARD) },
    now: "2026-11-20T12:00:00Z",
  };
  return processDocument(request);
}

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/link-card@0"\n      names: [link-card]\n---\n\n`;

const USE = `${IMPORT}## Read next\n\n:: link-card[The harbour at dusk] {href="harbour.comp.md" description="A walk along the old quay." width=20rem}\n`;

/** The text of `node`, its descendants' values joined. */
const textOf = (node: Node | undefined): string => node?.value ?? node?.children?.map(textOf).join("") ?? "";

/** The link card's box and the card in it, from the document's expanded tree. */
function linkCardOf(children: Node[]): { box: Node | undefined; card: Node | undefined } {
  const box = children.find((node) => node.type === "componentContainer");
  return { box, card: box?.children?.[0] };
}

describe("a link card", () => {
  it("is a box with stretch-link and the use's params, holding a card", () => {
    const { expanded, diagnostics } = processed(USE);
    expect(diagnostics).toEqual([]);
    const { box, card } = linkCardOf(expanded.children as Node[]);
    expect(box?.name).toBe("box");
    expect(box?.params).toEqual([
      { key: "stretch-link", breakpoint: null, value: true },
      { key: "width", breakpoint: null, value: "20rem" },
    ]);
    expect(card).toMatchObject({ type: "componentContainer", name: "box" });
  });

  it("has its title as a heading one level below the one above it, the heading's link the use's fallback", () => {
    const { card } = linkCardOf(processed(USE).expanded.children as Node[]);
    const [heading, description] = card?.children ?? [];
    expect(heading).toMatchObject({ type: "heading", depth: 3 });
    const link = heading?.children?.[0];
    expect(link).toMatchObject({ type: "link", url: "harbour.comp.md" });
    expect(textOf(link)).toBe("The harbour at dusk");
    expect(textOf(description)).toBe("A walk along the old quay.");
  });

  it("links to the document's own file, from a document in a folder too", () => {
    const { card } = linkCardOf(processed(USE, "posts/walks.comp.md").expanded.children as Node[]);
    expect(card?.children?.[0]?.children?.[0]).toMatchObject({ type: "link", url: "harbour.comp.md" });
  });

  it("is its title alone without a description", () => {
    const { expanded, diagnostics } = processed(`${IMPORT}:: link-card[Along the canal] {href="canal.comp.md"}\n`);
    expect(diagnostics).toEqual([]);
    const { card } = linkCardOf(expanded.children as Node[]);
    expect(card?.children?.map((node) => node.type)).toEqual(["heading"]);
  });

  it("reads in a plain Markdown renderer as its line, its title in it", () => {
    expect(plainHtml(USE)).toContain("<p>:: link-card[The harbour at dusk]");
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics: a card each, then three in a grid", () => {
    const { expanded, diagnostics } = processed(read(ROOT, "demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both"]);
    const grid = examples[3]?.document?.children.find((node) => node.type === "componentContainer");
    expect(grid?.children?.filter((node) => node.name === "box")).toHaveLength(3);
  });
});
