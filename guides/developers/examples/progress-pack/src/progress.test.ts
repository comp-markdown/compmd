import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./progress.ts";

describe("progress", () => {
  it("draws its value of its max, with its fallback as its label", () => {
    const shown = mountForTest(mount, { params: { value: 3, max: 4 }, fallback: "3 of 4 done" });
    const bar = shown.target.querySelector("progress");
    expect(bar?.value).toBe(3);
    expect(bar?.max).toBe(4);
    expect(bar?.getAttribute("aria-label")).toBe("3 of 4 done");
    expect(shown.target.textContent).toContain("3 of 4");
  });

  it("follows new params, keeps a value outside its range inside the bar, and goes when destroyed", () => {
    const shown = mountForTest(mount, { params: { value: 1, max: 10 }, fallback: "1 of 10" });
    shown.update({ value: 12, max: 10 });
    expect(shown.target.querySelector("progress")?.value).toBe(10);
    expect(shown.target.textContent).toContain("12 of 10");
    shown.destroy();
    expect(shown.target.querySelector("progress")).toBeNull();
  });
});
