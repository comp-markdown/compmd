// A framed use's sizer (docs/sizing-components.md, *The mechanism*): it fills the box's one cell and holds an invisible
// block of the component's natural size, so the box sizes as if it held the component, and the frame is placed over it.

export interface Sizer {
  /** Fills the box's cell; the frame goes in it, over all of it. */
  sizer: HTMLElement;
  /** The invisible block the component's natural size, which the frame reports. */
  natural: HTMLElement;
  /** Takes the sizer out of the box, and the box's cell back to the renderer's. */
  remove(): void;
}

/**
 * A sizer at the end of `box`, a host-language component's box (a grid of one column, `@compmd/render`), whose one row
 * it makes the box's whole inside; `span`s in an inline box.
 *
 * @example
 * const { sizer, natural } = sizerIn(box, false);
 * sizer.append(iframe);
 */
export function sizerIn(box: Element, inline: boolean): Sizer {
  const document = box.ownerDocument;
  // A component's box is an HTML element; anything else keeps its rows.
  const { style } = box as Partial<ElementCSSInlineStyle>;
  const tag = inline ? "span" : "div";
  const sizer = document.createElement(tag);
  sizer.className = "compmd-sizer";
  sizer.style.cssText = "display: block; position: relative; min-width: 0; min-height: 0; overflow: hidden;";
  const natural = sizer.appendChild(document.createElement(tag));
  natural.style.cssText = "display: block; visibility: hidden; width: 1px; height: 1px;";
  style?.setProperty("grid-template-rows", "minmax(0, 1fr)");
  box.append(sizer);
  return {
    sizer,
    natural,
    remove: () => {
      sizer.remove();
      style?.removeProperty("grid-template-rows");
    },
  };
}
