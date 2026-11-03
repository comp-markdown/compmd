import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./sparkline.ts";

describe("sparkline", () => {
  it("draws a line labelled with its fallback, and again in the other scheme", async () => {
    const shown = mountForTest(mount, { params: { values: [3, 5, 4, 8] }, fallback: "Visits rose all week" });
    await shown.settled();
    const image = shown.target.querySelector("[role=img]");
    expect(image?.getAttribute("aria-label")).toBe("Visits rose all week");
    expect(image?.querySelector("polyline")?.getAttribute("stroke")).toBe("#0b7a52");
    shown.setScheme("dark");
    await shown.settled();
    expect(shown.target.querySelector("polyline")?.getAttribute("stroke")).toBe("#7ee2b8");
    shown.destroy();
  });

  it("fails with what it can't draw, shows the fallback, and draws again at the next update", async () => {
    const shown = mountForTest(mount, { params: { values: [3, "lots"] }, fallback: "Visits rose all week" });
    await shown.settled();
    expect(shown.failures).toEqual(["a sparkline needs two or more numbers, and nothing else"]);
    expect(shown.target.textContent).toBe("Visits rose all week");
    shown.update({ values: [3, 5] });
    await shown.settled();
    expect(shown.target.querySelector("polyline")).not.toBeNull();
    shown.destroy();
  });
});
