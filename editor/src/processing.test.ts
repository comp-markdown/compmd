import { initSync, OpenDocumentJson, process, type Request } from "@compmd/processor";
import { createPage, type Page } from "@compmd/render";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { EXAMPLES, textOf } from "./documents.ts";
import type { Edit } from "./edit.ts";
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
  if (answer.type !== "processed") throw new Error(answer.message);
  const splices = answer.splices.map((splice) => JSON.parse(splice) as Parameters<Page["splice"]>[0][number]);
  const change = { text, diagnostics: [] };
  const [first] = splices;
  if (answer.opened && first) target.render({ type: "root", children: first.nodes, ...(first.inputs ? { inputs: first.inputs } : {}) }, change);
  else target.splice(splices, change);
}

/** A page's HTML, without the numbers of its boxes' classes, which go on counting as boxes render again. */
const html = (root: HTMLElement) => root.innerHTML.replaceAll(/compmd-b\d+/g, "compmd-b");

/** The page a fresh process of `text` gives. */
function fresh(base: Request, text: string): string {
  const { page: other, root } = page();
  other.render(process({ ...base, files: { ...base.files, [base.path]: text } }).expanded);
  return html(root);
}

const edited = (text: string, { start, end, text: inserted }: Edit) => text.slice(0, start) + inserted + text.slice(end);

describe("Processing", () => {
  it("gives a splice per edit, which a page applies to show what a new process gives", () => {
    const processing = new Processing(open);
    const { page: shown, root } = page();
    let text = "# Title\n\nOne.\n\n:: nope[N]\n";
    const opened = processing.handle({ type: "open", request: request(text) });
    expect(opened).toMatchObject({ type: "processed", opened: true, components: [] });
    apply(shown, opened, text);
    const edits = [
      { start: 7, end: 7, text: "!" },
      { start: 10, end: 10, text: "Zero.\n\n" },
    ];
    const answer = processing.handle({ type: "edits", edits });
    for (const edit of edits) text = edited(text, edit);
    expect(answer).toMatchObject({ type: "processed", opened: false, diagnostics: process(request(text)).diagnostics });
    expect(answer).not.toHaveProperty("components");
    expect(answer.type === "processed" && answer.splices).toHaveLength(2);
    apply(shown, answer, text);
    expect(html(root)).toBe(fresh(request(text), text));
  });

  for (const example of EXAMPLES) {
    it(`${example.name}: the page after each edit's splices is the page a new process gives`, () => {
      const base = { ...example.request, origins: true };
      const processing = new Processing(open);
      const { page: shown, root } = page();
      let text = textOf(example);
      apply(shown, processing.handle({ type: "open", request: base }), text);
      const middle = text.indexOf("\n\n", text.length / 2) + 2;
      // Each step's edits, in the text the step before left.
      const steps: ((now: string) => Edit[])[] = [
        () => [{ start: middle, end: middle, text: "x" }],
        () => [{ start: middle, end: middle + 1, text: "" }],
        () => [
          { start: middle, end: middle, text: "A new paragraph.\n\n" },
          { start: 0, end: 0, text: "Typed first.\n\n" },
        ],
        (now) => [{ start: now.length - 1, end: now.length - 1, text: "\n\nThe end." }],
      ];
      for (const step of steps) {
        const edits = step(text);
        const answer = processing.handle({ type: "edits", edits });
        for (const edit of edits) text = edited(text, edit);
        apply(shown, answer, text);
        expect(html(root), JSON.stringify(edits)).toBe(fresh(base, text));
      }
    });
  }

  it("gives the components when an edit changes them", () => {
    const processing = new Processing(open);
    const text = "```jsx component=hi runtime=react@19\nexport default () => 'hi';\n```\n";
    processing.handle({ type: "open", request: request(text) });
    const at = text.indexOf("'hi'") + 1;
    const answer = processing.handle({ type: "edits", edits: [{ start: at, end: at + 2, text: "yo" }] });
    expect(answer).toMatchObject({ components: [{ name: "hi", code: "export default () => 'yo';" }] });
  });

  it("fails without a document, and on a request it can't open", () => {
    const processing = new Processing(open);
    expect(processing.handle({ type: "edits", edits: [] })).toEqual({ type: "failed", message: "no document is open" });
    processing.handle({ type: "open", request: request("Text.\n") });
    expect(processing.handle({ type: "edits", edits: [] })).toMatchObject({ type: "processed", splices: [], diagnostics: [] });
    expect(processing.handle({ type: "open", request: { path: "a", now: "never" } })).toMatchObject({ type: "failed" });
    expect(processing.handle({ type: "edits", edits: [] })).toMatchObject({ type: "failed" });
  });
});
