import { describe, expect, it } from "vitest";
import { fitOf } from "./fit.ts";
import type { ExpandedParam } from "./tree.ts";

const params = (given: Record<string, unknown>): ExpandedParam[] => Object.entries(given).map(([key, value]) => ({ key, breakpoint: null, value }));

describe("fitOf: what a component gets as `fit` (spec §7.4)", () => {
  it("is contain, centred, for a box with neither param", () => {
    expect(fitOf([], false)).toEqual({ mode: "contain", x: 0.5, y: 0.5 });
  });

  it("takes each mode as written", () => {
    for (const mode of ["contain", "cover", "fill"]) expect(fitOf(params({ "object-fit": mode }), false).mode).toBe(mode);
  });

  it("reads two positions as the block one, then the inline one", () => {
    expect(fitOf(params({ "object-position": "end start" }), false)).toMatchObject({ x: 0, y: 1 });
    expect(fitOf(params({ "object-position": "start end" }), false)).toMatchObject({ x: 1, y: 0 });
    expect(fitOf(params({ "object-position": "center end" }), false)).toMatchObject({ x: 1, y: 0.5 });
  });

  it("reads one position as both", () => {
    expect(fitOf(params({ "object-position": "start" }), false)).toMatchObject({ x: 0, y: 0 });
    expect(fitOf(params({ "object-position": "end" }), false)).toMatchObject({ x: 1, y: 1 });
  });

  it("counts the inline position from the right in a right-to-left box, the block one from the top still", () => {
    expect(fitOf(params({ "object-position": "center start" }), true)).toMatchObject({ x: 1, y: 0.5 });
    expect(fitOf(params({ "object-position": "start end" }), true)).toMatchObject({ x: 0, y: 0 });
    expect(fitOf([], true)).toMatchObject({ x: 0.5, y: 0.5 });
  });

  it("takes the value written without a breakpoint, the only one a processor passes", () => {
    const given: ExpandedParam[] = [
      { key: "object-fit", breakpoint: "md", value: "fill" },
      { key: "object-fit", breakpoint: null, value: "cover" },
    ];
    expect(fitOf(given, false).mode).toBe("cover");
  });

  it("counts a value the processor wouldn't have passed as the default", () => {
    expect(fitOf(params({ "object-fit": "none", "object-position": "left top" }), false)).toEqual({ mode: "contain", x: 0.5, y: 0.5 });
    expect(fitOf(params({ "object-position": 3 }), false)).toMatchObject({ x: 0.5, y: 0.5 });
  });
});
