// Images whose origin the reader hasn't allowed (spec §15): each shows its alt text with a way to allow the origin, and
// is asked for again once the origin is allowed (docs/remote-images.md, decisions 5 and 7).

/** An image view that can ask its image source again. */
export interface Retrying {
  retry(): void;
}

/**
 * The page's refused images, by origin.
 *
 * @example
 * refused.add("https://photos.example.com", view);
 * refused.allowed("https://photos.example.com"); // each of its images asks again
 */
export class RefusedImages {
  readonly #byOrigin = new Map<string, Set<Retrying>>();

  /** `view`'s image, of `origin`, was refused. */
  add(origin: string, view: Retrying): void {
    let views = this.#byOrigin.get(origin);
    if (!views) this.#byOrigin.set(origin, (views = new Set()));
    views.add(view);
  }

  /** `view` shows something else now, or is gone. */
  remove(view: Retrying): void {
    for (const [origin, views] of this.#byOrigin) {
      views.delete(view);
      if (views.size === 0) this.#byOrigin.delete(origin);
    }
  }

  /** The reader allowed `origin`: each of its refused images asks its source again. */
  allowed(origin: string): void {
    const views = [...(this.#byOrigin.get(origin) ?? [])];
    this.#byOrigin.delete(origin);
    for (const view of views) view.retry();
  }
}
