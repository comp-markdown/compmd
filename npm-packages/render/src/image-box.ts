// An image box (spec §9.2): a box whose content is one paragraph holding one image, alone or as a link's only content,
// white space aside. Its image takes the box's inside and sits in it as the box's `object-fit` and `object-position`
// say, so the box's height crops it. The box is marked, its fit given as custom properties, and the base rules size the
// image (`IMAGE_BOX_RULES`).
import { fitOf, isRtl } from "./fit.ts";
import type { ExpandedNode, ExpandedParam } from "./tree.ts";

/** On an image box. */
export const IMAGE_BOX = "compmd-image-box";

const IMAGES = new Set(["image", "imageReference"]);
const LINKS = new Set(["link", "linkReference"]);

const isWhiteSpace = (node: ExpandedNode): boolean => node.type === "text" && typeof node.value === "string" && node.value.trim() === "";

/** Whether `inlines` are one image, or one link holding one image, white space aside. */
function isOneImage(inlines: readonly ExpandedNode[]): boolean {
  const shown = inlines.filter((node) => !isWhiteSpace(node));
  const [only] = shown;
  if (shown.length !== 1 || only === undefined) return false;
  if (IMAGES.has(only.type)) return true;
  return LINKS.has(only.type) && isOneImage(only.children ?? []);
}

/** Whether a box holding `children` is an image box. */
export function isImageBox(children: readonly ExpandedNode[]): boolean {
  const [paragraph] = children;
  return children.length === 1 && paragraph?.type === "paragraph" && isOneImage(paragraph.children ?? []);
}

/**
 * Marks `box` as an image box when `on`, its fit from `params` as custom properties its image reads; unmarks it
 * otherwise.
 *
 * @example
 * fitImage(box, node.params, isImageBox(node.children ?? []));
 */
export function fitImage(box: HTMLElement, params: readonly ExpandedParam[], on: boolean): void {
  box.classList.toggle(IMAGE_BOX, on);
  if (!on) {
    box.style.removeProperty("--compmd-image-fit");
    box.style.removeProperty("--compmd-image-position");
    return;
  }
  const fit = fitOf(params, isRtl(box));
  box.style.setProperty("--compmd-image-fit", fit.mode);
  box.style.setProperty("--compmd-image-position", `${String(fit.x * 100)}% ${String(fit.y * 100)}%`);
}

/**
 * The base rules of an image box: a grid of one cell, whose image fills the cell, so it takes the box's inside when
 * the box has a height and its own height at the box's width when it hasn't. At no specificity, so a host's own rules
 * win; on paper as on screen.
 */
export const IMAGE_BOX_RULES = `
:where(.${IMAGE_BOX}) { display: grid; grid-template-rows: minmax(0, 1fr); }
:where(.${IMAGE_BOX}) > p { margin: 0; min-block-size: 0; }
:where(.${IMAGE_BOX}) > p > a { display: block; block-size: 100%; }
:where(.${IMAGE_BOX}) img { display: block; inline-size: 100%; block-size: 100%; object-fit: var(--compmd-image-fit, contain); object-position: var(--compmd-image-position, center); }
`;
