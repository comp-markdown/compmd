// @vitest-environment happy-dom
import { describe, expect, it } from "vitest";
import { fallbackUntilDrawn } from "./fallback-until-drawn.ts";
import { sizerIn } from "./sizer.ts";

describe("fallbackUntilDrawn", () => {
  it("shows the fallback before the sizer, unseen and a pixel high, until the frame has drawn, once", () => {
    const box = document.createElement("div");
    const { sizer } = sizerIn(box, false);
    const fallback = document.createElement("span");
    fallback.textContent = "two of five";
    const drawn = fallbackUntilDrawn(box, sizer, fallback);
    expect([...box.children]).toEqual([fallback, sizer]);
    expect([sizer.style.visibility, sizer.style.height]).toEqual(["hidden", "1px"]);
    drawn();
    expect([...box.children]).toEqual([sizer]);
    expect([sizer.style.visibility, sizer.style.height]).toEqual(["", ""]);
    // A later call, from the frame loading again, changes nothing.
    box.append(fallback);
    drawn();
    expect([...box.children]).toEqual([sizer, fallback]);
  });
});
