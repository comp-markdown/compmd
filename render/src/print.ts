// Printing a page (docs/printing.md, decisions 1 and 3): paper is light, so the page is set light first, its components
// redraw for it, and the browser prints only once they have; then the page is as it was. A reader's own ⌘P can't wait
// like this: the print styles make it light, but a component still redrawing prints as it was.
import type { Schemes } from "./scheme.ts";

/**
 * Prints `root`'s page: sets it light, reads its scheme again so its components follow, waits for `drawn`, then calls
 * the window's `print()`, and puts the page's scheme back after it.
 *
 * @example
 * await printLight(root, schemes, () => surface.busy.idle());
 */
export async function printLight(root: HTMLElement, schemes: Schemes, drawn: () => Promise<void>): Promise<void> {
  const style = root.style;
  const before = { value: style.getPropertyValue("color-scheme"), priority: style.getPropertyPriority("color-scheme") };
  style.setProperty("color-scheme", "light", "important");
  try {
    schemes.read();
    await drawn();
    root.ownerDocument.defaultView?.print();
  } finally {
    if (before.value) style.setProperty("color-scheme", before.value, before.priority);
    else style.removeProperty("color-scheme");
    schemes.read();
  }
}
