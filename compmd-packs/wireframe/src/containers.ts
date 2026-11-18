// The parts that hold others (LANGUAGE.md, decision 5): the window's frame, and boxes laid out with CSS.

import { attribute, badge, childrenOf, element, flag, numberOf, textOf, type DrawPart } from "./dom.ts";
import type { Value } from "./lines.ts";

/** The dots of a desktop window's or a browser's bar. */
const dots = (document: Document) => element(document, "span", ["dots"], ...[1, 2, 3].map(() => element(document, "span", ["dot"])));

/** The frame's bar, by `device`: a title bar, a browser's bar with the title as its address, or a status bar. */
function bar(document: Document, device: string, title: string | undefined): HTMLElement {
  const text = element(document, "span", ["title"], title ?? "");
  if (device === "browser") return element(document, "div", ["bar", "browserbar"], dots(document), element(document, "span", ["address"], text));
  if (device === "phone" || device === "tablet") return element(document, "div", ["bar", "statusbar"], text);
  return element(document, "div", ["bar", "titlebar"], dots(document), text);
}

/** `window`: the frame, as wide as its block and as tall as its content. */
export const window: DrawPart = (part, drawing) => {
  const device = attribute(part, "device") ?? "desktop";
  const frame = element(drawing.document, "div", ["window"], bar(drawing.document, device, textOf(part)), element(drawing.document, "div", ["body"], ...childrenOf(part, drawing)));
  frame.dataset.device = device;
  return frame;
};

/** A box of `name` holding the part's children. */
const box =
  (name: string, tag = "div"): DrawPart =>
  (part, drawing) =>
    element(drawing.document, tag, [name, ...(flag(part, "large") ? ["large"] : [])], ...childrenOf(part, drawing));

export const header = box("header");
export const footer = box("footer");
export const panel = box("panel");

/** `section`: a titled box, its badge beside the title. */
export const section: DrawPart = (part, drawing) =>
  element(
    drawing.document,
    "div",
    ["section"],
    element(drawing.document, "div", ["section-title"], textOf(part) ?? "", badge(drawing.document, attribute(part, "badge"))),
    element(drawing.document, "div", ["section-body"], ...childrenOf(part, drawing)),
  );

const JUSTIFY: Readonly<Record<string, string>> = { start: "flex-start", between: "space-between", around: "space-around", end: "flex-end" };
const ALIGN: Readonly<Record<string, string>> = { left: "flex-start", center: "center", right: "flex-end" };

/** `row`: a flex row, placed along by `justify`, else by `align` (`justify` wins, as Wireloom's). */
export const row: DrawPart = (part, drawing) => {
  const drawn = element(drawing.document, "div", ["row"], ...childrenOf(part, drawing));
  drawn.style.justifyContent = JUSTIFY[attribute(part, "justify") ?? ""] ?? ALIGN[attribute(part, "align") ?? "left"] ?? "flex-start";
  return drawn;
};

/** A `col`'s `flex`: a fixed width, a share (`fr`), or the row's slack (`fill`, the default). */
function flexOf(width: Value | undefined): string {
  if (width?.kind !== "number") return "1 1 0";
  if (width.unit === "fr") return `${String(width.value)} 1 0`;
  return `0 0 ${String(width.value)}${width.unit === "" ? "px" : width.unit}`;
}

/** `col`: a flex column of its width. */
export const col: DrawPart = (part, drawing) => {
  const drawn = element(drawing.document, "div", ["col"], ...childrenOf(part, drawing));
  drawn.style.flex = flexOf(part.positionals[0]);
  return drawn;
};

/** `grid`: a CSS grid of `cols` and `rows`. */
export const grid: DrawPart = (part, drawing) => {
  const drawn = element(drawing.document, "div", ["grid"], ...childrenOf(part, drawing));
  drawn.style.gridTemplateColumns = `repeat(${String(numberOf(part, "cols") ?? 1)}, minmax(0, 1fr))`;
  drawn.style.gridTemplateRows = `repeat(${String(numberOf(part, "rows") ?? 1)}, auto)`;
  return drawn;
};

/** `cell`: at its `row` and `col`, its label over its children. */
export const cell: DrawPart = (part, drawing) => {
  const label = textOf(part);
  const drawn = element(drawing.document, "div", ["cell"], label === undefined ? undefined : element(drawing.document, "span", ["label"], label), ...childrenOf(part, drawing));
  drawn.style.gridRow = String(numberOf(part, "row") ?? 1);
  drawn.style.gridColumn = String(numberOf(part, "col") ?? 1);
  return drawn;
};
