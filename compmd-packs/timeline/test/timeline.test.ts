// The timeline as a host gets it: the pack's files, as a registry gives them, processed with a document that uses it.
// Its expanded tree (spec §12.3) is a `box` with a line on its start side, holding an `event` box for each event: a
// dot placed on the line, the date small and muted, the title in bold, then what the use wrote; or the list a quick
// timeline is. A plain Markdown reader shows each date, title and text in order (spec §4.5).
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

const IMPORT = `---\nmd:\n  import:\n    - from: "@comp-markdown/timeline@0"\n      names: [timeline, event]\n---\n\n`;

const EVENTS = `${IMPORT}::: timeline\n::: event {date=1897 title="The harbour wall"}\nBuilt in a summer.\n::: /event\n\n::: event {date=2024 tone=accent}\nThe lamps come back.\n::: /event\n::: /timeline\n`;

/** The text of `nodes`, their descendants' values joined. */
const textOf = (nodes: Node[] = []): string => nodes.map((node) => node.value ?? textOf(node.children)).join("");

const valueOf = (node: Node | undefined, key: string) => node?.params?.find((param) => param.key === key)?.value;

/** The timeline's box and its events, from a document's expanded tree. */
function timelineOf(text: string) {
  const { expanded, diagnostics } = processed(text);
  expect(diagnostics).toEqual([]);
  const [timeline] = expanded.children as Node[];
  return { timeline, events: (timeline?.children ?? []).filter((node) => node.name === "box") };
}

describe("a timeline", () => {
  it("is a box with a line on its start side, holding a box for each event", () => {
    const { timeline, events } = timelineOf(EVENTS);
    expect(timeline?.params).toEqual([
      { key: "border-inline-start", breakpoint: null, value: "strong" },
      { key: "padding-inline-start", breakpoint: null, value: "m" },
    ]);
    expect(events).toHaveLength(2);
  });

  it("gives each event a dot on the line, its date small and muted, its title in bold, then its text", () => {
    const [first] = timelineOf(EVENTS).events;
    const [dot, date, title, text] = first?.children ?? [];
    expect(valueOf(dot, "inset-inline-start")).toBe("-1.375rem");
    expect(valueOf(dot, "border-radius")).toBe("full");
    expect(valueOf(dot, "color")).toBe("muted");
    expect([valueOf(date, "color"), valueOf(date, "font-size"), textOf(date?.children)]).toEqual(["muted", "s", "1897"]);
    expect(title?.children?.[0]?.type).toBe("strong");
    expect(textOf(title?.children)).toBe("The harbour wall");
    expect(textOf(text?.children)).toBe("Built in a summer.");
  });

  it("fills the dot with the accent for tone=accent, and leaves the title out without one", () => {
    const [, second] = timelineOf(EVENTS).events;
    const [dot, date, text] = second?.children ?? [];
    expect(valueOf(dot, "color")).toBe("accent");
    expect(textOf(date?.children)).toBe("2024");
    expect(textOf(text?.children)).toBe("The lamps come back.");
    expect(second?.children).toHaveLength(3);
  });

  it("writes its events from data, one for each record", () => {
    const demo = processed(read("demo.comp.md"));
    const examples = (demo.expanded.children as (Node & { document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    const [timeline] = examples[3]?.document?.children ?? [];
    const events = (timeline?.children ?? []).filter((node) => node.name === "box");
    expect(events.map((event) => textOf(event.children?.[1]?.children))).toEqual(["March 2023", "October 2023", "April 2024"]);
  });

  it("is a list with no events, its line the same", () => {
    const { timeline } = timelineOf(`${IMPORT}::: timeline\n- **1897**: the harbour wall is built.\n::: /timeline\n`);
    expect(timeline?.children?.[0]?.type).toBe("list");
  });

  it("refuses a tone it hasn't with F009", () => {
    const { diagnostics } = processed(`${IMPORT}::: timeline\n::: event {date=1897 tone=loud}\nA.\n::: /event\n::: /timeline\n`);
    expect(diagnostics.map((diagnostic) => diagnostic.code)).toEqual(["F009"]);
  });

  it("reads in a plain Markdown renderer as its events' text, in order", () => {
    const html = plainHtml(EVENTS);
    expect(html.indexOf("Built in a summer.")).toBeGreaterThan(-1);
    expect(html.indexOf("Built in a summer.")).toBeLessThan(html.indexOf("The lamps come back."));
  });

  it("draws each of the demo's examples, its source shown too, with no diagnostics", () => {
    const { expanded, diagnostics } = processed(read("demo.comp.md"));
    expect(diagnostics).toEqual([]);
    const examples = (expanded.children as (Node & { render?: string; document?: { children: Node[] } })[]).filter((node) => node.type === "code" && node.document);
    expect(examples.map((example) => example.render)).toEqual(["both", "both", "both", "both", "both"]);
  });
});
