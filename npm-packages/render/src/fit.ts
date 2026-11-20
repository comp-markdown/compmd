// How a component fits what it draws in its box (spec §7.4, §9.2; docs/sizing-components.md, decision 1): its box's
// `object-fit` and `object-position`, resolved with the box's direction into the `fit` a component gets.
import type { Fit } from "./contract.ts";
import type { ExpandedParam } from "./tree.ts";

/** A fraction of the free space before the drawing, from the left or the top. */
type Place = Fit["x"];

const MODES: readonly Fit["mode"][] = ["contain", "cover", "fill"];

const PLACES: Readonly<Record<string, Place>> = { start: 0, center: 0.5, end: 1 };

/** The fit of a box with no `object-fit` or `object-position`: as large as fits, centred. */
export const CONTAIN: Fit = { mode: "contain", x: 0.5, y: 0.5 };

const base = (params: readonly ExpandedParam[], key: string): unknown => params.find((param) => param.key === key && param.breakpoint === null)?.value;

/**
 * The fit of a box with `params`, in a box that runs right to left when `rtl`. `object-position` is the block position
 * then the inline one, one value being both; the inline `start` is the right in a right-to-left box. A value the
 * processor wouldn't have passed counts as the default.
 *
 * @example
 * fitOf([{ key: "object-fit", breakpoint: null, value: "cover" }, { key: "object-position", breakpoint: null, value: "end start" }], true);
 * // { mode: "cover", x: 1, y: 1 }
 */
export function fitOf(params: readonly ExpandedParam[], rtl: boolean): Fit {
  const written = base(params, "object-fit");
  const mode = MODES.find((name) => name === written) ?? CONTAIN.mode;
  const position = base(params, "object-position");
  const [block = "center", inline = block] = typeof position === "string" ? position.trim().split(/\s+/) : [];
  const y = PLACES[block] ?? CONTAIN.y;
  const start = PLACES[inline] ?? CONTAIN.x;
  return { mode, x: rtl ? ((1 - start) as Place) : start, y };
}

/** Whether `element` runs right to left: its `dir`, or the nearest one above it, `auto` read from its text. */
export function isRtl(element: Element): boolean {
  const dir = element.closest("[dir]")?.getAttribute("dir")?.toLowerCase();
  if (dir !== "auto") return dir === "rtl";
  try {
    return element.matches(":dir(rtl)");
  } catch {
    return false;
  }
}

/** Whether two fits are the same. */
export const sameFit = (one: Fit, other: Fit): boolean => one.mode === other.mode && one.x === other.x && one.y === other.y;
