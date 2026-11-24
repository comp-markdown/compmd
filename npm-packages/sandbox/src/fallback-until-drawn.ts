// A framed use's fallback while its frame loads (spec §7.4, *A component not drawn yet*): the box shows the fallback,
// and the frame, unseen, loads behind it; once the frame says its component has mounted, the drawing takes the
// fallback's place in one change, so the text around an inline use moves at most once.

/**
 * Shows `fallback` in `box` before `sizer`, a framed use's sizer, which stays in the box unseen and one pixel high:
 * WebKit may stop rendering a frame with no area, and the frame must load and measure its component. Gives what puts
 * the drawing in the fallback's place, which does nothing after the first time.
 *
 * @example
 * const drawn = fallbackUntilDrawn(box, sizer, context.fallback.element);
 * child.on("rendered", drawn);
 */
export function fallbackUntilDrawn(box: Element, sizer: HTMLElement, fallback: Element): () => void {
  sizer.style.setProperty("visibility", "hidden");
  sizer.style.setProperty("height", "1px");
  box.insertBefore(fallback, sizer);
  let waiting = true;
  return () => {
    if (!waiting) return;
    waiting = false;
    fallback.remove();
    sizer.style.removeProperty("visibility");
    sizer.style.removeProperty("height");
  };
}
