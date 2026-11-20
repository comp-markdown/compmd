// A use frame's layout (docs/sizing-components.md, decisions 2 and 3, *The mechanism*): the component's box is the
// frame's size, the frame's document never scrolls, and the frame measures the component's natural size, which its
// parent gives the use's box.

/** On a frame's root once it renders a use (a `part`): what the rules below apply to. */
export const USE_FRAME = "compmd-use";
const NATURAL_HEIGHT = "compmd-natural-height";
const NATURAL_WIDTH = "compmd-natural-width";
/** The width of the box's inside, on the root: the frame's observer watches the body's attributes, and measuring must change none of them, or each measurement would start the next. */
const INSIDE = "--compmd-inside";

/**
 * The frame document's rules for a use. `#part` is a grid of one cell, the frame's size, so the component's box
 * fills it as the page's box's inside. To measure, the root gets no height from the frame (the height at the frame's
 * width), then `#part` is as wide as its content and the box a block (the width): WebKit otherwise gives the frame's own
 * height or width as a drawing's natural width.
 */
export const USE_FRAME_CSS = [
  `html.${USE_FRAME}, html.${USE_FRAME} body { height: 100%; overflow: hidden; }`,
  `html.${USE_FRAME} #part { display: grid; grid-template: minmax(0, 1fr) / minmax(0, 1fr); height: 100%; }`,
  `html.${USE_FRAME} #part > * { min-width: 0; min-height: 0; }`,
  `html.${NATURAL_HEIGHT}, html.${NATURAL_HEIGHT} body, html.${NATURAL_HEIGHT} #part { height: auto; }`,
  `html.${NATURAL_HEIGHT} #part { width: var(${INSIDE}, auto); }`,
  `html.${NATURAL_WIDTH} #part { width: max-content; }`,
  `html.${NATURAL_WIDTH} #part > * { display: block; }`,
].join(" ");

/**
 * The natural size of what `part` holds, laid out with no size from the frame, as CSS sizes content before it knows
 * its container: its height at `inside`, the width of the box's inside as the page last said (the frame's own width until it did),
 * then its width. Both in one task, so nothing is painted or observed in between, and neither depends on the frame's
 * height, so the frame and the box can't feed each other. The page's width, not the frame's: a browser may not give a
 * frame off screen its new size until it is shown.
 *
 * @example
 * const { width, height } = naturalSize(document.documentElement, part, 464);
 */
export function naturalSize(root: HTMLElement, part: HTMLElement, inside?: number): { width: number; height: number } {
  const target = part.firstElementChild ?? part;
  if (inside !== undefined) root.style.setProperty(INSIDE, `${String(inside)}px`);
  root.classList.add(NATURAL_HEIGHT);
  const height = target.getBoundingClientRect().height;
  root.classList.add(NATURAL_WIDTH);
  const width = target.getBoundingClientRect().width;
  root.classList.remove(NATURAL_HEIGHT, NATURAL_WIDTH);
  return { width, height };
}
