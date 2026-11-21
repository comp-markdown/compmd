// A wrapped component's props, from its package's types: only its own, each with its type, whether it is optional, its
// doc and its default.
import { rm } from "node:fs/promises";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { folder, write } from "../../test/folders.ts";
import { WRAPPED } from "../../test/wrapped.ts";
import type { Wrap } from "./config.ts";
import { entryOf, readProps } from "./props.ts";

let dir: string;

beforeAll(async () => {
  dir = await folder();
  await write(dir, WRAPPED);
});

afterAll(async () => {
  await rm(dir, { recursive: true, force: true });
});

const wrap = (from: string, kind: Wrap["kind"], exported = "default"): Wrap => ({ from, export: exported, kind, params: [] });

describe("readProps", () => {
  it("reads a React component's own props, not the DOM's, with their types, docs and defaults", () => {
    const props = readProps(dir, wrap("qr", "react"));
    expect(props.map((prop) => prop.name)).toEqual(["value", "level", "bgColor", "size", "title", "onDone", "when", "tags", "options", "label", "any"]);
    const byName = Object.fromEntries(props.map((prop) => [prop.name, prop]));
    expect(byName.value).toEqual({ name: "value", type: { kind: "string" }, optional: false, doc: "What the code holds" });
    expect(byName.level).toEqual({ name: "level", type: { kind: "literals", values: ["L", "M", "Q", "H"] }, optional: true, doc: "How much of it can be lost.", default: '"L"' });
    expect(byName.bgColor?.type).toEqual({ kind: "color" });
    expect(byName.title?.type).toEqual({ kind: "union", branches: [{ kind: "string" }, { kind: "function" }] });
    expect([byName.onDone, byName.when, byName.tags, byName.options].map((prop) => prop?.type.kind)).toEqual(["function", "date", "array", "object"]);
    expect(byName.any?.type).toEqual({ kind: "union", branches: [{ kind: "string" }, { kind: "number" }] });
  });

  it("reads a Svelte component's props by its export's name", () => {
    const props = readProps(dir, wrap("charts", "svelte", "BarChart"));
    expect(props.map((prop) => [prop.name, prop.type.kind, prop.default])).toEqual([
      ["data", "array", undefined],
      ["bandPadding", "number", "0.2"],
      ["height", "number", undefined],
    ]);
  });

  it("reads a custom element's own properties by its tag, not HTMLElement's", () => {
    expect(readProps(dir, wrap("viewer", "web-component", "x-viewer")).map((prop) => [prop.name, prop.type.kind])).toEqual([
      ["src", "string"],
      ["autoRotate", "boolean"],
      ["play", "function"],
    ]);
  });

  it("says what TypeScript said when it can't find the component", () => {
    expect(() => readProps(dir, wrap("qr", "react", "Missing"))).toThrow(/can't be read: .*Missing/);
    expect(() => readProps(dir, wrap("nowhere", "react"))).toThrow(/can't be read: .*nowhere/);
  });

  it("imports the component as its kind needs", () => {
    expect(entryOf(wrap("charts", "svelte", "BarChart"))).toContain('import { BarChart as Component } from "charts"');
    expect(entryOf(wrap("viewer", "web-component", "x-viewer"))).toContain('HTMLElementTagNameMap["x-viewer"]');
  });
});
