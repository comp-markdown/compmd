// The inputs a frame's uses bind, which are all its parent takes a `set` of.

import type { ExpandedComponent } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { chart } from "./channels.testing.ts";
import { boundInputs } from "./index.ts";

describe("boundInputs", () => {
  it("finds the inputs bound in a use, its slots and the components nested in them", () => {
    const tabs: ExpandedComponent = {
      type: "componentContainer",
      name: "tabs",
      params: [{ key: "open", breakpoint: null, value: 0, input: "tab" }],
      children: [chart],
      slots: [{ type: "componentSlot", name: "footer", children: [{ ...chart, params: [{ key: "x", breakpoint: null, value: 1, input: "year" }] }] }],
    };
    expect([...boundInputs([tabs])].sort()).toEqual(["region", "tab", "year"]);
  });

  it("is empty for uses that bind no input", () => {
    expect([...boundInputs([{ type: "componentLeaf", name: "legend", params: [{ key: "title", breakpoint: null, value: "Key" }] }])]).toEqual([]);
    expect([...boundInputs([])]).toEqual([]);
  });
});
