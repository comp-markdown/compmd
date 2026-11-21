// A component's box's type (spec §7.4, §9.2): read from the box's computed style, so it is the type the box shows,
// its own params and what it inherits from the page and the theme.
import type { TypeStyle } from "./contract.ts";

/**
 * The type `element` shows now.
 *
 * @example
 * typeStyleOf(box); // { family: "Georgia, serif", size: 14, weight: "700", style: "normal", color: "rgb(164, 14, 38)", lineHeight: "normal", letterSpacing: "normal" }
 */
export function typeStyleOf(element: Element): TypeStyle {
  const style = element.ownerDocument.defaultView?.getComputedStyle(element);
  return {
    family: style?.fontFamily ?? "",
    size: parseFloat(style?.fontSize ?? "") || 16,
    weight: style?.fontWeight ?? "400",
    style: style?.fontStyle ?? "normal",
    color: style?.color ?? "",
    lineHeight: style?.lineHeight ?? "normal",
    letterSpacing: style?.letterSpacing ?? "normal",
  };
}

/** Whether two types are the same. */
export const sameType = (one: TypeStyle, other: TypeStyle): boolean => (Object.keys(one) as (keyof TypeStyle)[]).every((key) => one[key] === other[key]);
