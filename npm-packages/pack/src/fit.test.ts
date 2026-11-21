// @vitest-environment happy-dom
import type { Fit } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { fillBox, fitBox } from "./fit.ts";

const svg = (attributes: Record<string, string> = {}) => {
  const element = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  for (const [name, value] of Object.entries(attributes)) element.setAttribute(name, value);
  return element;
};

describe("fitBox", () => {
  it.each<[Fit, string]>([
    [{ mode: "contain", x: 0.5, y: 0.5 }, "xMidYMid meet"],
    [{ mode: "contain", x: 0, y: 1 }, "xMinYMax meet"],
    [{ mode: "cover", x: 1, y: 0 }, "xMaxYMin slice"],
    [{ mode: "fill", x: 0, y: 0 }, "none"],
  ])("gives an SVG the box's size, scaled as %o says", (fit, aspect) => {
    const drawing = fitBox(svg({ viewBox: "0 0 8 8" }), fit);
    expect(drawing.getAttribute("preserveAspectRatio")).toBe(aspect);
    expect([drawing.style.display, drawing.style.width, drawing.style.height]).toEqual(["block", "100%", "100%"]);
  });

  it("is contain, centred, without a fit", () => {
    expect(fitBox(svg({ viewBox: "0 0 8 8" })).getAttribute("preserveAspectRatio")).toBe("xMidYMid meet");
  });

  it("gives an SVG with a width and a height and no viewBox one, so it can scale, and keeps one it has", () => {
    expect(fitBox(svg({ width: "120", height: "80" })).getAttribute("viewBox")).toBe("0 0 120 80");
    expect(fitBox(svg({ width: "120", height: "80", viewBox: "0 0 12 8" })).getAttribute("viewBox")).toBe("0 0 12 8");
    expect(fitBox(svg({ width: "100%" })).hasAttribute("viewBox")).toBe(false);
  });

  it("keeps a max-width the drawing has, and places it in the box's width as the fit says", () => {
    const drawing = svg({ viewBox: "0 0 8 8", style: "max-width: 400px" });
    expect(fitBox(drawing).style.maxWidth).toBe("400px");
    const margins = (x: Fit["x"]) => {
      const { style } = fitBox(svg({ viewBox: "0 0 8 8" }), { mode: "contain", x, y: 0.5 });
      return [style.marginLeft, style.marginRight];
    };
    expect([margins(0), margins(0.5), margins(1)]).toEqual([["0px", "auto"], ["auto", "auto"], ["auto", "0px"]]);
  });

  it.each(["img", "canvas", "video"])("fits an %s by CSS's object-fit and object-position", (name) => {
    const drawing = fitBox(document.createElement(name), { mode: "cover", x: 1, y: 0.5 });
    expect([drawing.style.width, drawing.style.height, drawing.style.objectFit, drawing.style.objectPosition]).toEqual(["100%", "100%", "cover", "100% 50%"]);
  });

  it("leaves content that flows as it is", () => {
    const text = fitBox(document.createElement("p"), { mode: "cover", x: 0, y: 0 });
    expect(text.getAttribute("style")).toBeNull();
  });
});

describe("fillBox", () => {
  it("fills the box and contains its size, CSS's 300 by 150 its natural size", () => {
    const chart = fillBox(document.createElement("div"));
    expect([chart.style.width, chart.style.height, chart.style.contain]).toEqual(["100%", "100%", "size"]);
    expect(chart.style.getPropertyValue("contain-intrinsic-size")).toBe("300px 150px");
  });
});
