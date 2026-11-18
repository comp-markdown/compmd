import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./__component__.ts";

describe("__component__", () => {
  it("draws the block's code, follows new params and the scheme, and goes when destroyed", () => {
    const drawn = mountForTest(mount, { params: { code: "first\nsecond" }, fallback: "Two lines of __language__" });
    const figure = () => drawn.target.querySelector<HTMLElement>("[role=img]");
    expect(figure()?.getAttribute("aria-label")).toBe("Two lines of __language__");
    expect(figure()?.querySelectorAll("div")).toHaveLength(2);
    drawn.update({ code: "only" });
    expect(figure()?.querySelectorAll("div")).toHaveLength(1);
    drawn.setScheme("dark");
    expect(figure()?.style.background).toBe("#1f2328");
    drawn.destroy();
    expect(drawn.target.children).toHaveLength(0);
  });
});
