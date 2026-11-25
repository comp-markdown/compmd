import { readFileSync } from "node:fs";
import { join } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { initSync as initComponents, readProps } from "./components.ts";
import { initSync, OpenDocument, process, type Request } from "./index.ts";

beforeAll(() => {
  initSync({ module: readFileSync(join(import.meta.dirname, "../pkg/compmd_wasm_bg.wasm")) });
  initComponents({ module: readFileSync(join(import.meta.dirname, "../pkg/components/compmd_components_wasm_bg.wasm")) });
});

/** A badge whose code takes `label`, `tone` and `emphasis`, and whose signature declares `label`, `tone` and `size`. */
const BADGE = [
  "```jsx component=badge runtime=react@19",
  "export default function Badge({ label, tone, emphasis }) {",
  "  return <b className={tone}>{label}</b>;",
  "}",
  "```",
  "",
  "```compmd component=badge signature",
  "params:",
  "  label: string",
  "  tone: info | warn = info",
  "  size: number?",
  "```",
  "",
  ':: badge[New] {label="New"}',
  "",
].join("\n");

const request = (text: string): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now: "2026-10-22T09:00:00Z" });

/** The W104s, each `line:column message`. */
const w104 = (diagnostics: readonly { code: string; message: string; range: { start: { line: number; column: number } } }[]) =>
  diagnostics.filter(({ code }) => code === "W104").map(({ message, range }) => `${String(range.start.line)}:${String(range.start.column)} ${message}`);

describe("readProps", () => {
  it("reads a React or a Svelte component's props, with their types where the source states them", () => {
    expect(readProps("jsx", "export default function Badge({ label, ...more }) { return null; }")).toEqual({ props: [{ name: "label" }], rest: true });
    expect(readProps("svelte", '<script lang="ts">\n  let { value }: { value: number } = $props();\n</script>')).toEqual({
      props: [{ name: "value", type: { kind: "number" } }],
      rest: false,
    });
  });

  it("reads nothing it can't, nor a language without props", () => {
    expect(readProps("jsx", "export default function Badge(props) { return null; }")).toBeUndefined();
    expect(readProps("js", "export default () => null;")).toBeUndefined();
    expect(readProps("vue", "<template></template>")).toBeUndefined();
  });
});

describe("component analysis", () => {
  it("is W104 where a component and its signature disagree, at the param or the fence, given a reader", () => {
    expect(w104(process(request(BADGE), { componentReader: readProps }).diagnostics)).toEqual([
      "11:3 the signature declares `size`, which a.comp.md has no prop for (`size`)",
      "7:1 a.comp.md has the prop `emphasis`, which the signature doesn't declare",
    ]);
  });

  it("is nothing without a reader", () => {
    expect(w104(process(request(BADGE)).diagnostics)).toEqual([]);
  });

  it("follows an open document's edits", () => {
    const document = new OpenDocument(request(BADGE), { componentReader: readProps });
    expect(w104(document.diagnostics)).toHaveLength(2);
    const at = BADGE.indexOf(", emphasis");
    document.edit(at, at + ", emphasis".length, ", size");
    expect(w104(document.diagnostics)).toEqual([]);
    document.free();
  });
});
