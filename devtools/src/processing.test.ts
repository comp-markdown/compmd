import { initSync, OpenDocumentJson, process, type Request } from "@compmd/processor";
import { createPage, type Page } from "@compmd/render";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { Processing } from "./processing.ts";
import type { FromWorker } from "./protocol.ts";

beforeAll(() => {
  const wasm = createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm");
  initSync({ module: readFileSync(wasm) });
});

const request = (text: string): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-10-22T09:00:00Z", origins: true });
const open = (r: Request) => new OpenDocumentJson(r);

const roots: HTMLElement[] = [];
afterEach(() => {
  for (const root of roots.splice(0)) root.remove();
});

/** A page in the document, as the frame has one. */
function page(): { page: Page; root: HTMLElement } {
  const root = document.body.appendChild(document.createElement("main"));
  roots.push(root);
  return { page: createPage(root), root };
}

/** What the frame does with an answer: the first splice after opening renders as a whole tree. */
function apply(target: Page, answer: FromWorker, text: string): void {
  if (answer.type === "failed") throw new Error(answer.message);
  if (answer.type !== "processed") throw new Error("a hint, not a change");
  const splices = answer.splices.map((splice) => JSON.parse(splice) as Parameters<Page["splice"]>[0][number]);
  const change = { text, diagnostics: [] };
  const [first] = splices;
  if (answer.opened && first) target.render({ type: "root", children: first.nodes, ...(first.inputs ? { inputs: first.inputs } : {}) }, change);
  else target.splice(splices, change);
}

/** A page's HTML, without the numbers of its boxes' classes, which go on counting as boxes render again. */
const html = (root: HTMLElement) => root.innerHTML.replaceAll(/compmd-b\d+/g, "compmd-b");

describe("Processing, with remote data", () => {
  it("reports the remote URLs a document needs once typing pauses, each once, for the page to ask about and fetch", async () => {
    const reported: string[][] = [];
    const processing = new Processing(open, { onRemote: (urls) => reported.push(urls), pause: 5 });
    const text = "---\nmd:\n  data:\n    - from: https://api.example.com/a.json\n      as: a\n---\n\nText.\n";
    processing.handle({ type: "open", request: request(text), caret: 0 });
    await vi.waitFor(() => {
      expect(reported).toEqual([["https://api.example.com/a.json"]]);
    });
    // The document opened again with the data, and a new source typed in: only the new one is reported.
    processing.handle({ type: "open", request: request(text), caret: 0 });
    const at = text.indexOf("---\n\nText");
    processing.handle({ type: "edits", edits: [{ start: at, end: at, text: "    - from: https://api.example.com/b.json\n      as: b\n" }], caret: 0 });
    await vi.waitFor(() => {
      expect(reported).toEqual([["https://api.example.com/a.json"], ["https://api.example.com/b.json"]]);
    });
  });
});

describe("Processing", () => {
  it("gives a splice per edit, which a page applies to show what a new process gives", () => {
    const processing = new Processing(open);
    const { page: shown, root } = page();
    let text = "# Title\n\nOne.\n\n:: nope[N]\n";
    const opened = processing.handle({ type: "open", request: request(text), caret: 0 });
    expect(opened).toMatchObject({ type: "processed", opened: true, components: [], inputs: {} });
    apply(shown, opened, text);
    const edits = [
      { start: 7, end: 7, text: "!" },
      { start: 10, end: 10, text: "Zero.\n\n" },
    ];
    const answer = processing.handle({ type: "edits", edits, caret: 0 });
    for (const { start, end, text: inserted } of edits) text = text.slice(0, start) + inserted + text.slice(end);
    expect(answer).toMatchObject({ type: "processed", opened: false, diagnostics: process(request(text)).diagnostics });
    expect(answer).not.toHaveProperty("components");
    expect(answer.type === "processed" && answer.splices).toHaveLength(2);
    apply(shown, answer, text);
    const { page: fresh, root: freshRoot } = page();
    fresh.render(process(request(text)).expanded);
    expect(html(root)).toBe(html(freshRoot));
  });

  it("gives the inputs with their values", () => {
    const processing = new Processing(open);
    const answer = processing.handle({ type: "open", request: { ...request("---\nmd:\n  inputs:\n    n: number = 2\n---\n"), inputs: { n: 5 } }, caret: 0 });
    expect(answer).toMatchObject({ inputs: { n: { type: "number", value: 5 } } });
  });

  it("gives the components when an edit changes them", () => {
    const processing = new Processing(open);
    const text = "```jsx component=hi runtime=react@19\nexport default () => 'hi';\n```\n";
    processing.handle({ type: "open", request: request(text), caret: 0 });
    const at = text.indexOf("'hi'") + 1;
    const answer = processing.handle({ type: "edits", edits: [{ start: at, end: at + 2, text: "yo" }], caret: 0 });
    expect(answer).toMatchObject({ components: [{ name: "hi", code: "export default () => 'yo';" }] });
  });

  it("fails without a document, and on a request it can't open", () => {
    const processing = new Processing(open);
    expect(processing.handle({ type: "edits", edits: [], caret: 0 })).toEqual({ type: "failed", message: "no document is open" });
    expect(processing.handle({ type: "hint", caret: 0 })).toEqual({ type: "hint" });
    processing.handle({ type: "open", request: request("Text.\n"), caret: 0 });
    expect(processing.handle({ type: "edits", edits: [], caret: 0 })).toMatchObject({ type: "processed", splices: [], diagnostics: [] });
    expect(processing.handle({ type: "open", request: { path: "a", now: "never" }, caret: 0 })).toMatchObject({ type: "failed" });
    expect(processing.handle({ type: "edits", edits: [], caret: 0 })).toMatchObject({ type: "failed" });
  });

  it("gives the hint at the caret of the text the edits left, or alone when only the caret moved", () => {
    const processing = new Processing(open);
    const text = "::: box {border=subtle}\nB.\n::: /box\n";
    const opened = processing.handle({ type: "open", request: request(text), caret: 18 });
    expect(opened).toMatchObject({ hint: { subject: { kind: "param", value: "subtle" } } });
    const typed = processing.handle({ type: "edits", edits: [{ start: 16, end: 22, text: "strong" }], caret: 22 });
    expect(typed).toMatchObject({ hint: { subject: { value: "strong" } } });
    expect(processing.handle({ type: "hint", caret: 5 })).toMatchObject({ type: "hint", hint: { subject: { kind: "component", name: "box" } } });
    expect(processing.handle({ type: "hint", caret: 26 })).toEqual({ type: "hint" });
  });
});
