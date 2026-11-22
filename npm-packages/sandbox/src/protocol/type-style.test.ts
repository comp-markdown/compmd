import { describe, expect, it } from "vitest";
import { frameFontsIn } from "./type-style.ts";

describe("the fonts a type-style message brings", () => {
  const file = new Blob(["wOF2"]);

  it("are each a face the page declared, with a weight, a style and a file", () => {
    const font = { name: "compmd-font-3-harbour-serif", weight: "bold", style: "italic", file };
    expect(frameFontsIn([font])).toEqual([font]);
  });

  it("leave out what isn't one, so nothing a frame declares reads as more than a face", () => {
    const wrong = [
      { name: "Georgia", weight: "normal", style: "normal", file },
      { name: 'compmd-font-1-x"; src: url(x)', weight: "normal", style: "normal", file },
      { name: "compmd-font-1-x", weight: "600", style: "normal", file },
      { name: "compmd-font-1-x", weight: "normal", style: "oblique", file },
      { name: "compmd-font-1-x", weight: "normal", style: "normal", file: "not a file" },
      null,
    ];
    expect(frameFontsIn(wrong)).toEqual([]);
    expect(frameFontsIn(undefined)).toEqual([]);
  });
});
