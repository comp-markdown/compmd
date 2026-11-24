import { readFileSync } from "node:fs";
import { initSync, type Signature } from "@compmd/processor";
import * as svelte from "svelte/compiler";
import ts from "typescript";
import { beforeAll, describe, expect, it } from "vitest";
import { analysePack, mismatches } from "./analysis.ts";
import type { Pack } from "./packs.ts";
import type { Compilers } from "./props.ts";

const compilers: Compilers = { ts, svelte };

beforeAll(() => {
  initSync({ module: readFileSync(new URL(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** A signature with `params`, as `name: type` pairs, and `slots`. */
function signature(params: Record<string, Signature["params"][number]["type"]>, slots: Signature["slots"] = null): Signature {
  return { params: Object.entries(params).map(([name, type]) => ({ name, type, optional: false, default: null, doc: null })), slots, headings: "relative" };
}

describe("mismatches", () => {
  it("finds a declared param with no prop, a prop not declared, and a slot with no prop", () => {
    const found = mismatches("Badge.jsx", signature({ label: "string", size: "number" }, ["default", "side-panel"]), { props: [{ name: "label" }, { name: "emphasis" }, { name: "children" }], rest: false });
    expect(found).toEqual([
      { param: "size", message: "the signature declares `size`, which Badge.jsx has no prop for (`size`)" },
      { message: "the signature declares the slot `side-panel`, which Badge.jsx has no prop for (`sidePanel`)" },
      { message: "Badge.jsx has the prop `emphasis`, which the signature doesn't declare" },
    ]);
  });

  it("names params and slots as the bindings do, and takes the binding's own prop", () => {
    const props = { props: [{ name: "alignItems" }, { name: "sidePanel" }, { name: "compmd" }], rest: false };
    expect(mismatches("Grid.tsx", signature({ "align-items": "string" }, ["side-panel"]), props)).toEqual([]);
  });

  it("checks a prop's type where the source states one", () => {
    const props = {
      props: [
        { name: "count", type: { kind: "string" as const } },
        { name: "tone", type: { kind: "enum" as const, members: ["info"] } },
        { name: "title", type: { kind: "string" as const } },
        { name: "pad", type: { kind: "string" as const } },
        { name: "bound" },
      ],
      rest: false,
    };
    const declared = signature({ count: "number", tone: { enum: ["info", "warn"] }, title: { enum: ["a", "b"] }, pad: "space", bound: { input: "number" } });
    expect(mismatches("Card.tsx", declared, props).map(({ message }) => message)).toEqual([
      "the signature declares `count` as a number, which Card.tsx's prop takes as a string",
      "the signature declares `tone` as one of info, warn, which Card.tsx's prop takes as one of info",
    ]);
  });

  it("takes any declared param when the component takes props it doesn't name", () => {
    expect(mismatches("Box.jsx", signature({ a: "string" }, ["default"]), { props: [], rest: true })).toEqual([]);
  });
});

/** A pack with `files`, its document `index.comp.md`. */
function pack(files: Record<string, string>): Pack {
  const components = Object.keys(files)
    .filter((path) => /\.(jsx|tsx|svelte)$/.test(path))
    .map((path) => ({ path, language: path.slice(path.lastIndexOf(".") + 1) as "jsx" | "tsx" | "svelte" }));
  return { name: "@kit/badge", version: "1.0.0", dir: "/kit", document: "index.comp.md", files, assets: [], unlisted: [], components, size: 0 };
}

describe("analysePack", () => {
  it("checks a definition fence's component, as the conformance case has it, at the fence and at the param", () => {
    const document = [
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
    ].join("\n");
    expect(analysePack(pack({ "index.comp.md": document }), {}, compilers)).toEqual([
      "@kit/badge@1.0.0/index.comp.md:11:3: W104 the signature declares `size`, which index.comp.md has no prop for (`size`)",
      "@kit/badge@1.0.0/index.comp.md:7:1: W104 index.comp.md has the prop `emphasis`, which the signature doesn't declare",
    ]);
  });

  it("checks an imported component file by the name its import gives it, and leaves one it can't read", () => {
    const document = "---\nmd:\n  import:\n    - from: ./Card.svelte\n      names: [{name: Card, as: card}]\n    - from: ./Gauge.jsx\n      names: [{name: Gauge, as: gauge}]\n---\n\n```compmd component=card signature\nparams:\n  title: string\nslots: [default]\n```\n\n```compmd component=gauge signature\nparams:\n  value: number\n```\n";
    const files = { "index.comp.md": document, "Card.svelte": "<script>\n  let { title } = $props();\n</script>\n", "Gauge.jsx": "export default function Gauge(props) { return null; }" };
    const md = { import: [{ from: "./Card.svelte", names: [{ name: "Card", as: "card" }] }, { from: "./Gauge.jsx", names: [{ name: "Gauge", as: "gauge" }] }] };
    expect(analysePack(pack(files), md, compilers)).toEqual(["@kit/badge@1.0.0/index.comp.md:10:1: W104 the signature declares the slot `default`, which Card.svelte has no prop for (`children`)"]);
  });
});
