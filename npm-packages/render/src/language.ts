// A document's language and direction (spec §2.1, §12.3): the root's `lang` and `dir`, set on the element it renders
// into, the page's or an example's box. A root that has neither leaves the element as it was, so a document that
// writes neither keeps where it is shown's; an attribute this set before and the root no longer has is removed.
import type { ExpandedRoot } from "./tree.ts";

const NAMES = ["lang", "dir"] as const;

/** The attributes each element was given by a root, so a later root without them takes them away. */
const given = new WeakMap<Element, Set<string>>();

/**
 * Sets `root`'s `lang` and `dir` on `element`, and removes those an earlier root set that this one hasn't.
 *
 * @example
 * setLanguage(area, { type: "root", children: [], lang: "ar", dir: "rtl" }); // <div lang="ar" dir="rtl">
 */
export function setLanguage(element: Element, root: Pick<ExpandedRoot, "lang" | "dir">): void {
  const mine = given.get(element) ?? new Set<string>();
  for (const name of NAMES) {
    const value = root[name];
    if (value !== undefined) {
      element.setAttribute(name, value);
      mine.add(name);
    } else if (mine.delete(name)) {
      element.removeAttribute(name);
    }
  }
  given.set(element, mine);
}
