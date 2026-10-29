import { initSync, OpenDocument, process, type Request } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { Processing } from "./processing.ts";

beforeAll(() => {
  const wasm = createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm");
  initSync({ module: readFileSync(wasm) });
});

const request = (text: string): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-10-22T09:00:00Z", origins: true });

describe("Processing", () => {
  const open = (r: Request) => new OpenDocument(r);

  it("gives the tree a new process gives, after opening and after each batch of edits", () => {
    const processing = new Processing(open);
    const text = "# Title\n\nOne.\n\n:: nope[N]\n";
    const opened = processing.handle({ type: "open", request: request(text) });
    expect(opened).toMatchObject({ type: "processed", components: [] });
    const edits = [
      { start: 7, end: 7, text: "!" },
      { start: 10, end: 10, text: "Zero.\n\n" },
    ];
    const answer = processing.handle({ type: "edits", edits });
    const after = "# Title!\n\nZero.\n\nOne.\n\n:: nope[N]\n";
    const fresh = process(request(after));
    expect(answer).toMatchObject({ type: "processed", expanded: fresh.expanded, diagnostics: fresh.diagnostics });
    expect(answer).not.toHaveProperty("components");
  });

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
    expect(processing.handle({ type: "open", request: { path: "a", now: "never" } })).toMatchObject({ type: "failed" });
    expect(processing.handle({ type: "edits", edits: [] })).toMatchObject({ type: "failed" });
  });
});
