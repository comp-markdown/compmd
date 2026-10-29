// A series of times, for the status line and the bench: the last one and the nearest-rank 95th percentile.

export class Timings {
  readonly #values: number[] = [];

  add(ms: number): void {
    this.#values.push(ms);
  }

  get count(): number {
    return this.#values.length;
  }

  get last(): number | undefined {
    return this.#values.at(-1);
  }

  /** The nearest-rank 95th percentile; `undefined` before any. */
  get p95(): number | undefined {
    if (this.#values.length === 0) return undefined;
    const sorted = [...this.#values].sort((a, b) => a - b);
    return sorted[Math.ceil(sorted.length * 0.95) - 1];
  }

  /** Every time, in order. */
  get values(): readonly number[] {
    return this.#values;
  }
}
