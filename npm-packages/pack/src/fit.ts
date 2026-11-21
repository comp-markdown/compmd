// A drawing in its component's box (docs/sizing-components.md, decisions 1 and 3): `fitBox` for one with an aspect
// ratio of its own, by the box's `fit` (spec §7.4); `fillBox` for one with none, sized in script from its box.

import type { Fit } from "@compmd/render";

/** A box with no `object-fit` or `object-position`: as large as fits, centred. */
export const CONTAIN: Fit = { mode: "contain", x: 0.5, y: 0.5 };

const ALIGN = ["Min", "Mid", "Max"] as const;

/** `fit` as an SVG's `preserveAspectRatio`: contain is `meet`, cover `slice`, fill `none`. */
const aspectOf = (fit: Fit): string => (fit.mode === "fill" ? "none" : `x${ALIGN[fit.x * 2] ?? "Mid"}Y${ALIGN[fit.y * 2] ?? "Mid"} ${fit.mode === "cover" ? "slice" : "meet"}`);

const styleOf = (element: Element): CSSStyleDeclaration | undefined => (element as Partial<ElementCSSInlineStyle>).style;

/** An SVG's `viewBox` from its `width` and `height`, when it has none: without one it can't scale. */
function giveViewBox(svg: Element): void {
  if (svg.hasAttribute("viewBox")) return;
  const [width, height] = [Number(svg.getAttribute("width")), Number(svg.getAttribute("height"))];
  if (width > 0 && height > 0) svg.setAttribute("viewBox", `0 0 ${String(width)} ${String(height)}`);
}

/**
 * Fits `element`, a drawing with an aspect ratio of its own, in its component's box as `fit` says: it takes the box's
 * size, and an SVG scales by its `preserveAspectRatio`, an `img`, a `canvas` or a `video` by CSS's `object-fit` and
 * `object-position`. In a box with no height, it is as tall as its ratio makes it at the box's width. A `max-width` it
 * has is kept, so a drawing that says its largest size stays within it, placed in the box's width as the fit says
 * (centred by default). Any other element is left as it is: content that flows fits no box.
 *
 * @example
 * export default component((target, { fit }) => { target.append(fitBox(qrCode(), fit)); });
 */
export function fitBox<E extends Element>(element: E, fit: Fit = CONTAIN): E {
  const style = styleOf(element);
  const name = element.localName;
  if (!style || !["svg", "img", "canvas", "video"].includes(name)) return element;
  style.setProperty("display", "block");
  style.setProperty("width", "100%");
  style.setProperty("height", "100%");
  // Narrower than the box (its `max-width`), it sits where the fit says: the free space before it, from the left.
  style.setProperty("margin-left", fit.x === 0 ? "0px" : "auto");
  style.setProperty("margin-right", fit.x === 1 ? "0px" : "auto");
  if (name === "svg") {
    giveViewBox(element);
    element.setAttribute("preserveAspectRatio", aspectOf(fit));
  } else {
    style.setProperty("object-fit", fit.mode);
    style.setProperty("object-position", `${String(fit.x * 100)}% ${String(fit.y * 100)}%`);
  }
  return element;
}

/**
 * Makes `element`, which holds a drawing with no aspect ratio that a script sizes from it (a chart, a map), fill its
 * component's box, and contain its size: what it drew last never counts as its natural size, which is CSS's 300 by
 * 150 pixels, so a box that was stretched can shrink again.
 *
 * @example
 * const chart = fillBox(document.createElement("div"));
 * new ResizeObserver(() => drawChart(chart, chart.clientWidth, chart.clientHeight)).observe(chart);
 */
export function fillBox<E extends Element>(element: E): E {
  const style = styleOf(element);
  if (!style) return element;
  style.setProperty("width", "100%");
  style.setProperty("height", "100%");
  style.setProperty("contain", "size");
  style.setProperty("contain-intrinsic-size", "300px 150px");
  return element;
}
