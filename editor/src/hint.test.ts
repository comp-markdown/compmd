import type { HintSubject, ParameterHint, PositionalHint } from "@compmd/processor";
import { describe, expect, it } from "vitest";
import { lineOf, runs, type HintLine } from "./hint.ts";

const range = { start: { line: 1, column: 1 }, end: { line: 1, column: 2 } };
const hintOf = (subject: HintSubject, completions: PositionalHint["completions"] = []): PositionalHint => ({ range, subject, completions, diagnostics: [] });

/** The line's parts as text, `**` around the name and backticks around code. */
function written(line: HintLine | undefined): string[] {
  return (line?.parts ?? []).map((part) => part.map((run) => (run.name ? `**${run.text}**` : run.code ? `\`${run.text}\`` : run.text)).join(""));
}

const border: ParameterHint = {
  name: "border",
  origin: "common",
  takes: { tokens: ["none", "subtle", "strong"], forms: ["border"], examples: ["1px solid #d0d7de"], upTo: 1, flag: false, quoted: true, description: "`none`, `subtle`, `strong` or a CSS border" },
  required: false,
  breakpoints: true,
};

describe("lineOf", () => {
  it("says what a param takes, on its key, with its default, doc and breakpoint", () => {
    const tone: ParameterHint = { ...border, name: "tone", origin: "declared", default: "info", doc: "The badge's color.", required: true, breakpoints: false };
    expect(written(lineOf(hintOf({ kind: "param", component: "box", param: border })))).toEqual([
      "**border** on `box`, a common param",
      "takes `none`, `subtle`, `strong` or a CSS border",
      "takes a breakpoint, `md:border=…`",
    ]);
    expect(written(lineOf(hintOf({ kind: "param", component: "badge", param: tone })))).toEqual([
      "**tone** on `badge`",
      "takes `none`, `subtle`, `strong` or a CSS border",
      "default `info`",
      "required",
      "The badge's color.",
    ]);
    const lang = { ...border, name: "lang", origin: "attribute" as const, breakpoints: false };
    expect(written(lineOf(hintOf({ kind: "param", component: "box", param: lang })))[0]).toBe("**lang** on `box`, an attribute");
  });

  it("offers a value's tokens, the written one marked, and says the forms it takes too", () => {
    const completions = ["none", "subtle", "strong"].map((text) => ({ label: text, text }));
    const line = lineOf(hintOf({ kind: "param", component: "box", param: border, value: "subtle" }, completions));
    expect(line?.values).toEqual(completions);
    expect(line?.current).toBe("subtle");
    expect(written(line)).toEqual(["**border** on `box`, a common param", "takes `none`, `subtle`, `strong` or a CSS border"]);
    const only = { ...border, takes: { ...border.takes, forms: [] } };
    expect(written(lineOf(hintOf({ kind: "param", component: "box", param: only, value: "" }, completions))), "tokens alone").toEqual(["**border** on `box`, a common param"]);
  });

  it("names a component, where it comes from, its params and slots", () => {
    const signature = {
      params: [
        { name: "kind", type: { enum: ["note", "warning"] }, optional: false, default: { kind: "bare" as const, value: "note" }, doc: null },
        { name: "of", type: { input: "date" }, optional: true, default: null, doc: null },
        { name: "meta", type: "object", optional: false, default: { kind: "yaml" as const, value: { a: 1 } }, doc: null },
      ],
      slots: ["default", { name: "rows", each: "items", as: "as" }],
      headings: "relative" as const,
    };
    const own = { kind: "compmd", provided: false, signature, defined: range };
    expect(written(lineOf(hintOf({ kind: "component", name: "card", component: own })))).toEqual([
      "**card**, a CompMD component defined in this document (line 1)",
      "params `kind: note | warning = note`, `of: input<date>?`, `meta: object = {\"a\":1}`",
      "slots `default`, `rows`",
    ]);
    const core = { kind: "processor", file: "@compmd/core@1.0.0/index.comp.md", provided: true, signature: { ...signature, params: [], slots: null } };
    expect(written(lineOf(hintOf({ kind: "component", name: "box", component: core })))).toEqual(["**box**, a built-in from `@compmd/core`", "no slots"]);
    const chart = { kind: "jsx", file: "Chart.jsx", provided: false };
    expect(written(lineOf(hintOf({ kind: "component", name: "chart", component: chart })))).toEqual(["**chart**, a `jsx` component from `Chart.jsx`", "no signature: its params go unchecked"]);
    expect(written(lineOf(hintOf({ kind: "component", name: "card", component: { kind: "compmd", provided: false } })))[0]).toBe("**card**, a CompMD component defined in this document");
    expect(written(lineOf(hintOf({ kind: "component", name: "nope" })))).toEqual(["**nope** stands for no component here"]);
  });

  it("lists the params a list takes, and a container's slots", () => {
    expect(written(lineOf(hintOf({ kind: "params", component: "box", params: [border, { ...border, name: "gap" }] })))).toEqual(["`box` takes `border`, `gap`"]);
    expect(written(lineOf(hintOf({ kind: "slot", component: "card", slot: "footer", slots: ["default", "footer"] })))).toEqual(["slot **footer** of `card`", "`card` has slots `default`, `footer`"]);
    expect(written(lineOf(hintOf({ kind: "slot", component: "note", slot: "x", slots: [] })))[1]).toBe("`note` declares no slots");
  });

  it("says what a path names and what a function does", () => {
    const name = { name: "books", type: "list(map(string, dyn))", about: "data from `./books.csv`", fields: ["pages: double"] };
    expect(written(lineOf(hintOf({ kind: "path", path: "md.data.books", name })))).toEqual([
      "**md.data.books**",
      "data from `./books.csv`",
      "`list(map(string, dyn))`",
      "fields `pages: double`",
    ]);
    expect(written(lineOf(hintOf({ kind: "path", path: "nope", name: { name: "nope" } })))).toEqual(["**nope**", "nothing here has this name"]);
    const hint = { signatures: ["list.exists(x, predicate) → bool"], doc: "true when any element satisfies the predicate" };
    expect(written(lineOf(hintOf({ kind: "function", name: "exists", hint })))).toEqual(["**exists**", "`list.exists(x, predicate) → bool`", hint.doc]);
    expect(written(lineOf(hintOf({ kind: "function", name: "nope" })))).toEqual(["**nope** is no function of the profile"]);
  });

  it("says what a frontmatter key holds, and offers the type names", () => {
    expect(written(lineOf(hintOf({ kind: "frontmatter", path: ["md", "tz"], about: "the time zone of `md.today()`" })))).toEqual(["**md.tz**", "the time zone of `md.today()`"]);
    expect(written(lineOf(hintOf({ kind: "frontmatter", path: ["md", "nope"] })))).toEqual(["**md.nope**"]);
    const types = [{ label: "number", text: "number" }];
    const number = lineOf(hintOf({ kind: "type", name: "number", takes: { ...border.takes, description: "a number, such as `12`" } }, types));
    expect(written(number)).toEqual(["type **number**", "a number, such as `12`"]);
    expect(number?.values).toEqual(types);
    expect(number?.current).toBe("number");
    expect(written(lineOf(hintOf({ kind: "type", name: "nu" }, types)))[1]).toBe("no such type");
  });

  it("has nothing to say without a subject", () => {
    expect(lineOf({ range, completions: [], diagnostics: [] })).toBeUndefined();
  });
});

describe("runs", () => {
  it("makes backtick spans code", () => {
    expect(runs("takes `none` or a length")).toEqual([{ text: "takes " }, { text: "none", code: true }, { text: " or a length" }]);
    expect(runs("`a`")).toEqual([{ text: "a", code: true }]);
  });
});
