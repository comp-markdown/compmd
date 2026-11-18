// What the components of a page or region are still drawing (`context.busy`, spec §7.4): a page has drawn once every
// drawing its components gave has settled, or has had `BUSY_MS`, so one that never settles can't hold it forever.

/** How long a drawing a component gave `busy` holds its page, at most. */
export const BUSY_MS = 10_000;

/** The drawings in progress, and those waiting for them to end. */
export class Busy {
  readonly #drawing = new Set<Promise<void>>();
  #waiting: (() => void)[] = [];

  /** A component is drawing until `work` settles, fulfilled or rejected, or `BUSY_MS` pass. */
  add(work: Promise<unknown>): void {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const capped = Promise.race([
      work.then(
        () => undefined,
        () => undefined,
      ),
      new Promise<void>((resolve) => {
        timer = setTimeout(resolve, BUSY_MS);
      }),
    ]);
    this.#drawing.add(capped);
    void capped.then(() => {
      clearTimeout(timer);
      this.#drawing.delete(capped);
      if (this.#drawing.size === 0) this.#done();
    });
  }

  /** Resolves once nothing is drawing: at once when nothing is, after the drawings added meanwhile too. */
  idle(): Promise<void> {
    if (this.#drawing.size === 0) return Promise.resolve();
    return new Promise((resolve) => this.#waiting.push(resolve));
  }

  #done(): void {
    const waiting = this.#waiting;
    this.#waiting = [];
    for (const resolve of waiting) resolve();
  }
}
