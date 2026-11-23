import { describe, expect, it } from "vitest";
import { followsDirection, gradientFor } from "./gradient.ts";

describe("gradientFor", () => {
  it("makes a linear gradient's logical sides physical for the text's direction", () => {
    const gradient = "linear-gradient(to block-end inline-end, red, blue)";
    expect(gradientFor(gradient, "ltr")).toBe("linear-gradient(to bottom right, red, blue)");
    expect(gradientFor(gradient, "rtl")).toBe("linear-gradient(to bottom left, red, blue)");
    expect(gradientFor("repeating-linear-gradient(to inline-start, red 0 1rem, blue 1rem 2rem)", "rtl")).toBe("repeating-linear-gradient(to right, red 0 1rem, blue 1rem 2rem)");
  });

  it("keeps an angle in left-to-right text and mirrors it in right-to-left text", () => {
    const gradient = "linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)";
    expect(gradientFor(gradient, "ltr")).toBe(gradient);
    expect(gradientFor(gradient, "rtl")).toBe("linear-gradient(calc(360deg - 315deg), transparent 1.4rem, #fff176 1.4rem)");
  });

  it("makes a radial gradient's logical position physical, its shape and size kept", () => {
    expect(gradientFor("radial-gradient(circle at inline-start block-start, #fff5d6, transparent 60%)", "rtl")).toBe("radial-gradient(circle at right top, #fff5d6, transparent 60%)");
    expect(gradientFor("radial-gradient(4rem at inline-end, red, blue)", "ltr")).toBe("radial-gradient(4rem at right, red, blue)");
  });

  it("keeps stops as written, a color's own angle and commas among them", () => {
    const gradient = "linear-gradient(to inline-end, hsl(120deg 50% 50%), color-mix(in srgb, red, blue))";
    expect(gradientFor(gradient, "rtl")).toBe("linear-gradient(to left, hsl(120deg 50% 50%), color-mix(in srgb, red, blue))");
  });

  it("leaves physical sides, a gradient with no head, and any other value as they are", () => {
    for (const value of ["linear-gradient(to bottom left, red, blue)", "linear-gradient(red, blue)", "radial-gradient(red, blue)", "#fff176", "subtle"]) {
      expect(gradientFor(value, "rtl")).toBe(value);
    }
  });
});

describe("followsDirection", () => {
  it("is true for a logical side or an angle, and false for physical sides, no head, or a color", () => {
    expect(followsDirection("linear-gradient(to block-end, red, blue)")).toBe(true);
    expect(followsDirection("linear-gradient(90deg, red, blue)")).toBe(true);
    expect(followsDirection("radial-gradient(at inline-start, red, blue)")).toBe(true);
    expect(followsDirection("linear-gradient(to bottom, red, blue)")).toBe(false);
    expect(followsDirection("radial-gradient(circle, red, blue)")).toBe(false);
    expect(followsDirection("linear-gradient(hsl(120deg 50% 50%), blue)")).toBe(false);
    expect(followsDirection("#fff176")).toBe(false);
  });
});
