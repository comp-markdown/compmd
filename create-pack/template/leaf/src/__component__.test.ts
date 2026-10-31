// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import mount from "./__component__.ts";

describe("__component__", () => {
  it("draws its value, follows new params, and goes when destroyed", () => {
    const target = document.createElement("div");
    const fallback = { text: "Hello, as text", element: document.createElement("p") };
    const mounted = mount(target, { params: { value: "Hello" }, fallback });
    expect(target.textContent).toBe("Hello");
    expect(target.querySelector("[aria-label]")?.getAttribute("aria-label")).toBe("Hello, as text");
    mounted.update({ value: "Again" });
    expect(target.textContent).toBe("Again");
    mounted.destroy();
    expect(target.textContent).toBe("");
  });
});
