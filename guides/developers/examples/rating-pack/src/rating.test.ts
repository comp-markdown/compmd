import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./rating.ts";

describe("rating", () => {
  it("draws its value of its max, and sets the bound input when a star is chosen", () => {
    const shown = mountForTest(mount, { params: { value: 3, max: 5, size: "small" }, fallback: "3 of 5 stars", inputs: ["value"] });
    const stars = () => [...shown.target.querySelectorAll("button")];
    expect(stars().map((star) => star.textContent).join("")).toBe("★★★☆☆");
    stars()[4]?.click();
    expect(shown.sets).toEqual([{ param: "value", value: 5 }]);
    shown.update({ value: 5, max: 5, size: "large", color: "#b8860b" });
    expect(stars().map((star) => star.textContent).join("")).toBe("★★★★★");
    shown.destroy();
  });
});
