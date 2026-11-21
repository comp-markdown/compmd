import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount, { MOST, starsOf } from "./rating.ts";

/** The drawing, its label, how many stars it has and how wide its filled part is, in stars. */
function drawn(target: Element) {
  const drawing = target.querySelector("svg");
  return {
    label: drawing?.getAttribute("aria-label"),
    role: drawing?.getAttribute("role"),
    stars: drawing?.querySelectorAll("g[fill-opacity] path").length,
    filled: Number(drawing?.querySelector("clipPath rect")?.getAttribute("width")) / 24,
    viewBox: drawing?.getAttribute("viewBox"),
  };
}

describe("rating", () => {
  it("draws `of` stars, `value` of them filled, labelled by the fallback", () => {
    const shown = mountForTest(mount, { params: { value: 4, of: 5 }, fallback: "Four of five stars" });
    expect(drawn(shown.target)).toEqual({ label: "Four of five stars", role: "img", stars: 5, filled: 4, viewBox: "0 0 120 24" });
    shown.destroy();
    expect(shown.target.childElementCount).toBe(0);
  });

  it("says “Rated 3.5 of 10” with no fallback, and follows new params", () => {
    const shown = mountForTest(mount, { params: { value: 3.5, of: 10 }, fallback: "" });
    expect(drawn(shown.target)).toMatchObject({ label: "Rated 3.5 of 10", stars: 10, filled: 3.5 });
    shown.update({ value: 2, of: 5 });
    expect(drawn(shown.target)).toMatchObject({ label: "Rated 2 of 5", stars: 5, filled: 2 });
    shown.destroy();
  });

  it("fits its box as the document says", () => {
    const shown = mountForTest(mount, { params: { value: 1, of: 5 } });
    expect(shown.target.querySelector("svg")?.getAttribute("preserveAspectRatio")).toBe("xMidYMid meet");
    shown.setFit({ mode: "contain", x: 0, y: 0.5 });
    expect(shown.target.querySelector("svg")?.getAttribute("preserveAspectRatio")).toBe("xMinYMid meet");
    shown.destroy();
  });

  it("gives each rating on a page its own clip", () => {
    const [a, b] = [mountForTest(mount, { params: { value: 1, of: 5 } }), mountForTest(mount, { params: { value: 2, of: 5 } })];
    const id = (target: Element) => target.querySelector("clipPath")?.id;
    expect(id(a.target)).not.toBe(id(b.target));
    a.destroy();
    b.destroy();
  });
});

describe("starsOf", () => {
  it("rounds the value to the nearest half, between 0 and `of`, and keeps `of` a whole number from 1 to the most", () => {
    expect([starsOf({ value: 3.3, of: 5 }), starsOf({ value: 3.8, of: 5 }), starsOf({ value: -1, of: 5 }), starsOf({ value: 9, of: 5 })]).toEqual([
      { value: 3.5, of: 5 },
      { value: 4, of: 5 },
      { value: 0, of: 5 },
      { value: 5, of: 5 },
    ]);
    expect([starsOf({ value: 1, of: 0 }), starsOf({ value: 1, of: 3.6 }), starsOf({ value: 1, of: 500 }), starsOf({ value: Number.NaN, of: Number.NaN })]).toEqual([
      { value: 1, of: 1 },
      { value: 1, of: 4 },
      { value: 1, of: MOST },
      { value: 0, of: 5 },
    ]);
  });
});
