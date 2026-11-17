// Printing a view (docs/printing.md, decisions 1 and 3), in either page mode: the container is set light, the page and
// every frame follow it, and the browser prints once the view has settled, every frame redrawn and sized; then the
// container's scheme is put back and followed again.

/**
 * Prints the view in `container`: sets it light, calls `follow` so the page and its frames take the scheme, waits for
 * `settled`, then calls the window's `print()`; puts the scheme back and calls `follow` again after it.
 *
 * @example
 * await printLight(container, followScheme, () => settling.whenSettled());
 */
export async function printLight(container: HTMLElement, follow: () => void, settled: () => Promise<void>): Promise<void> {
  const style = container.style;
  const before = { value: style.getPropertyValue("color-scheme"), priority: style.getPropertyPriority("color-scheme") };
  style.setProperty("color-scheme", "light", "important");
  try {
    follow();
    await settled();
    container.ownerDocument.defaultView?.print();
  } finally {
    if (before.value) style.setProperty("color-scheme", before.value, before.priority);
    else style.removeProperty("color-scheme");
    follow();
  }
}
