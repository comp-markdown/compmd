// `stretch-link` (spec §9.2): a box whose first link covers it, as a card that is a link is used. CSS can't select a
// box's first link among its descendants, so the box marks it, and the base rules stretch the marked link's `::after`
// over the box, which is positioned. The box's other links are above it, so they stay links.
import type { ExpandedParam } from "./tree.ts";

/** On a box with `stretch-link`. */
export const STRETCHES = "compmd-stretch-link";
/** On that box's first link. */
export const STRETCHED = "compmd-stretched";

/** Whether a box with `params` has `stretch-link`; it takes no breakpoint. */
export const stretches = (params: readonly ExpandedParam[]): boolean => params.some((param) => param.key === "stretch-link" && param.breakpoint === null && param.value === true);

/**
 * Marks `box` and its first link when `on`, and unmarks a link it marked before, which an update may have moved.
 *
 * @example
 * stretchFirstLink(card, stretches(node.params));
 */
export function stretchFirstLink(box: HTMLElement, on: boolean): void {
  box.classList.toggle(STRETCHES, on);
  const first = on ? box.querySelector("a[href]") : null;
  for (const marked of box.querySelectorAll(`.${STRETCHED}`)) if (marked !== first) marked.classList.remove(STRETCHED);
  first?.classList.add(STRETCHED);
}
