// A wrapped package's settings, `compmd.wrap.json`: read when there, what is wrong with them said.
import { rm } from "node:fs/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { folder, write } from "../../test/folders.ts";
import { problemOfWrap, readWrap } from "./config.ts";

let dir: string;

beforeEach(async () => {
  dir = await folder();
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

const valid = { from: "qr", export: "default", kind: "react", params: ["value"] };

describe("readWrap", () => {
  it("reads the settings, and nothing for a pack that wraps no package", async () => {
    expect(readWrap(dir)).toBeUndefined();
    await write(dir, { "compmd.wrap.json": { ...valid, label: "title", input: { value: "onChange" } } });
    expect(readWrap(dir)).toEqual({ ...valid, label: "title", input: { value: "onChange" } });
  });

  it("says when the file isn't JSON, or isn't a wrap's settings", async () => {
    await write(dir, { "compmd.wrap.json": "{" });
    expect(() => readWrap(dir)).toThrow(/compmd.wrap.json isn't JSON/);
    await write(dir, { "compmd.wrap.json": { ...valid, kind: "vue" } });
    expect(() => readWrap(dir)).toThrow("compmd.wrap.json: `kind` is one of react, svelte, web-component, library");
  });
});

describe("problemOfWrap", () => {
  it("names the first key that is wrong", () => {
    const problems = [[], { ...valid, from: 1 }, { ...valid, export: undefined }, { ...valid, params: "value" }, { ...valid, overrides: { x: 1 } }, { ...valid, slots: { default: 1 } }, { ...valid, box: 3 }].map(problemOfWrap);
    expect(problems).toEqual([
      "it isn't a JSON object",
      "`from` names the npm package it wraps",
      '`export` names the component: "default", or the export\'s name',
      "`params` lists the props that are params, by name",
      "`overrides` holds an object for each param it changes",
      "`slots` maps names to names",
      "`box` is a string",
    ]);
    expect(problemOfWrap(valid)).toBeUndefined();
  });
});
