// When what a page or region shows moved, for a host that draws beside it (a gutter of line numbers): after a render
// is painted, when the root or a block in it changes size, and when fonts finish loading. At most once an animation
// frame, and only for a caller that listens.

/** Calls `onLayout` when what `root` shows may have moved. */
export class Layout {
  readonly #root: HTMLElement;
  readonly #onLayout: () => void;
  readonly #resizes: ResizeObserver | undefined;
  readonly #observed = new Set<Element>();
  readonly #fonts: FontFaceSet | undefined;
  readonly #fontsLoaded = () => {
    this.changed();
  };
  #frame: number | undefined;

  constructor(root: HTMLElement, onLayout: () => void) {
    this.#root = root;
    this.#onLayout = onLayout;
    const view = root.ownerDocument.defaultView;
    this.#resizes = view?.ResizeObserver
      ? new view.ResizeObserver(() => {
          this.changed();
        })
      : undefined;
    this.#resizes?.observe(root);
    // Not every DOM a page renders in has fonts to load (a test's).
    this.#fonts = "fonts" in root.ownerDocument ? root.ownerDocument.fonts : undefined;
    this.#fonts?.addEventListener("loadingdone", this.#fontsLoaded);
  }

  /** What `root` shows changed: its blocks are observed, and `onLayout` is called once it is painted. */
  rendered(): void {
    const blocks = new Set(this.#root.children);
    for (const block of this.#observed) {
      if (blocks.has(block)) continue;
      this.#resizes?.unobserve(block);
      this.#observed.delete(block);
    }
    for (const block of blocks) {
      if (this.#observed.has(block)) continue;
      this.#resizes?.observe(block);
      this.#observed.add(block);
    }
    this.changed();
  }

  /** Something may have moved: `onLayout` at the next animation frame, once however often it is asked. */
  changed(): void {
    const view = this.#root.ownerDocument.defaultView;
    if (this.#frame !== undefined || !view) return;
    this.#frame = view.requestAnimationFrame(() => {
      this.#frame = undefined;
      this.#onLayout();
    });
  }

  destroy(): void {
    if (this.#frame !== undefined) this.#root.ownerDocument.defaultView?.cancelAnimationFrame(this.#frame);
    this.#frame = undefined;
    this.#resizes?.disconnect();
    this.#observed.clear();
    this.#fonts?.removeEventListener("loadingdone", this.#fontsLoaded);
  }
}
