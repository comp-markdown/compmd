import { describe, expect, it } from "vitest";
import { lookOf, placeOf } from "./wheel.ts";

describe("placeOf", () => {
  const tops = [-500, -100, 300, 900];

  it("is the section that has reached the line, and how far through it", () => {
    expect(placeOf(tops, 100, 1500)).toBe(1.5);
    expect(placeOf(tops, 300, 1500)).toBe(2);
    expect(placeOf(tops, 1200, 1500)).toBe(3);
    expect(placeOf([-500, -100, 300], 100, 1500)).toBe(1.5);
  });

  it("is 0 before the first section, and at most the last's end", () => {
    expect(placeOf([200, 800], 100, 1200)).toBe(0);
    expect(placeOf([-900, -600], 100, 0)).toBe(1);
  });
});

describe("lookOf", () => {
  it("shows the reader's section whole, and the others smaller and fainter the further they are", () => {
    expect(lookOf(0)).toEqual({ offset: 0, scale: 1, opacity: 1 });
    const near = lookOf(-1);
    const far = lookOf(3);
    expect(near.offset).toBe(-1);
    expect(far.scale).toBeLessThan(near.scale);
    expect(far.opacity).toBeLessThan(near.opacity);
    expect(lookOf(9)).toMatchObject({ scale: lookOf(4).scale, opacity: lookOf(4).opacity });
  });
});
