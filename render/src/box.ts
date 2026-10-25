// A block's outer box (spec §9.2): its layout and common params as CSS, and `class`, `id`, `lang` and `dir`.
import { attributeOf, blockCss, type ParentLayout, type Stylesheet } from "./styles.ts";
import type { ExpandedParam } from "./tree.ts";

/** The classes each element got from its `class` param, so the next params can take them away again. */
const authorClasses = new WeakMap<Element, string[]>();

/**
 * Styles `element` as the box of a block with `params`, in a parent that lays it out as `parent`; `layout` for a
 * `box`, whose layout params style it too. The caller flushes the stylesheet.
 */
export function applyBox(element: HTMLElement, params: readonly ExpandedParam[], parent: ParentLayout, stylesheet: Stylesheet, layout = false): void {
  element.classList.add(stylesheet.style(element, blockCss(params, parent, layout)));
  const classes = attributeOf(params, "class")?.split(/\s+/).filter(Boolean) ?? [];
  const before = authorClasses.get(element) ?? [];
  element.classList.remove(...before.filter((name) => !classes.includes(name)));
  element.classList.add(...classes);
  authorClasses.set(element, classes);
  for (const name of ["id", "lang", "dir"]) {
    const value = attributeOf(params, name);
    if (value === undefined) element.removeAttribute(name);
    else if (element.getAttribute(name) !== value) element.setAttribute(name, value);
  }
}
