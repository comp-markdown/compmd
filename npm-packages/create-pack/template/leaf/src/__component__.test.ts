import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./__component__.ts";

describe("__component__", () => {
  it("draws its value, follows new params, and goes when destroyed", () => {
    const shown = mountForTest(mount, { params: { value: "Hello" }, fallback: "Hello, as text" });
    expect(shown.target.textContent).toBe("Hello");
    expect(shown.target.querySelector("[aria-label]")?.getAttribute("aria-label")).toBe("Hello, as text");
    shown.update({ value: "Again" });
    expect(shown.target.textContent).toBe("Again");
    shown.destroy();
    expect(shown.target.textContent).toBe("");
  });

  it("fits its box as the document says: contain by default, and the box's object-fit and object-position", () => {
    const shown = mountForTest(mount, { params: { value: "Hello" } });
    const drawing = () => shown.target.querySelector("svg");
    expect(drawing()?.getAttribute("preserveAspectRatio")).toBe("xMidYMid meet");
    shown.setFit({ mode: "cover", x: 0, y: 0.5 });
    expect(drawing()?.getAttribute("preserveAspectRatio")).toBe("xMinYMid slice");
    shown.destroy();
  });
});
