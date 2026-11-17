import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./wireframe.ts";

describe("wireframe", () => {
  it("draws the block's code, follows new params and the scheme, and goes when destroyed", () => {
    const drawn = mountForTest(mount, { params: { code: "first\nsecond" }, fallback: "Two lines of wireframe" });
    const figure = () => drawn.target.querySelector<HTMLElement>("[role=img]");
    expect(figure()?.getAttribute("aria-label")).toBe("Two lines of wireframe");
    expect(figure()?.querySelectorAll("div")).toHaveLength(2);
    drawn.update({ code: "only" });
    expect(figure()?.querySelectorAll("div")).toHaveLength(1);
    drawn.setScheme("dark");
    expect(figure()?.style.background).toBe("#1f2328");
    drawn.destroy();
    expect(drawn.target.children).toHaveLength(0);
  });
});
