// Which inputs the page's frame may set (S16): an input of the document, by its own key, that a use sent binds.

import type { ExpandedComponent, ExpandedRoot, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { PageInputs } from "./page-inputs.ts";

/** A use binding its param `value` to `input`. */
const binding = (input: string): ExpandedComponent => ({ type: "componentLeaf", name: "slider", params: [{ key: "value", breakpoint: null, value: "", input }], fallback: [] });

const inputs = { region: { type: "string" as const, value: "eu" }, secret: { type: "string" as const, value: "" } };

const tree = (children: ExpandedComponent[]): ExpandedRoot => ({ type: "root", children, inputs });

describe("PageInputs", () => {
  it("takes an input the document declares and a use sent binds, and no other", () => {
    const page = new PageInputs();
    page.rendered(tree([binding("region")]));
    expect(page.binds("region")).toBe(true);
    expect(page.binds("secret"), "declared, but no use binds it").toBe(false);
    expect(page.binds("other"), "bound by none, declared by none").toBe(false);
  });

  it("never takes a name every object has: an input is the document's own key", () => {
    const page = new PageInputs();
    page.rendered(tree([binding("__proto__"), binding("constructor")]));
    expect(page.binds("__proto__")).toBe(false);
    expect(page.binds("constructor")).toBe(false);
  });

  it("follows the uses a splice brings, as an object or as JSON text, and the inputs it gives", () => {
    const page = new PageInputs();
    page.rendered(tree([]));
    const splice: Splice = { from: 0, to: 0, nodes: [binding("secret")], lines: 0, offset: 0 };
    page.spliced([JSON.stringify(splice)]);
    expect(page.binds("secret")).toBe(true);
    page.spliced([], {});
    expect(page.binds("secret"), "the document declares no inputs now").toBe(false);
  });
});
