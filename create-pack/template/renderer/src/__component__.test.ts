// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import mount from "./__component__.ts";

describe("__component__", () => {
  it("draws the block's code, follows new params, and goes when destroyed", () => {
    const target = document.createElement("div");
    const fallback = { text: "Two lines of __language__", element: document.createElement("pre") };
    const mounted = mount(target, { params: { code: "first\nsecond", theme: "light" }, fallback });
    expect(target.querySelector("[role=img]")?.getAttribute("aria-label")).toBe("Two lines of __language__");
    expect(target.querySelectorAll("[role=img] div")).toHaveLength(2);
    mounted.update({ code: "only", theme: "dark" });
    expect(target.querySelectorAll("[role=img] div")).toHaveLength(1);
    mounted.destroy();
    expect(target.children).toHaveLength(0);
  });
});
