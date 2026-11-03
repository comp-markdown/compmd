// What the host's page keeps of what its observer and its frames report (docs/devtools.md, O2): the last 2,000 events of
// each source, the host's page itself and each of its frames, with what each one reported marked as its report (O6). A
// frame that floods its reports pushes out only its own.

import type { FrameId, Observed } from "./protocol/observed.ts";

/** Marks `events` as the report of the frame `by`, or leaves them as what the host's page saw itself. */
export const reportedBy = (events: readonly Observed[], by?: FrameId): ObservedEvent[] => (by === undefined ? [...events] : events.map((event) => ({ ...event, by })));

/**
 * An event as the host's page keeps it: `by` is the frame of the host's page whose report it is, which it or a frame
 * under it recorded; an event without `by` is what the host's page saw itself.
 */
export type ObservedEvent = Observed & { by?: FrameId };

/** What a view's observation keeps; `view.observe` makes one. */
export interface Observation {
  /** What is kept, in the order it came: the last 2,000 events of the host's page, and of each of its frames. */
  readonly events: readonly ObservedEvent[];
  /** How many events were dropped, by source: `""` for the host's page's own, a frame's id for its reports. */
  readonly dropped: Readonly<Record<FrameId, number>>;
  /** Stops listening; the sandbox stops observing once no observation listens. */
  stop(): void;
}

/** The events kept of each source, each with its place in the order they came. */
export class Log {
  readonly #limit: number;
  readonly #kept = new Map<FrameId, { order: number; event: ObservedEvent }[]>();
  readonly #dropped: Record<FrameId, number> = {};
  #order = 0;
  #merged: readonly ObservedEvent[] | undefined;

  constructor(limit = 2000) {
    this.#limit = limit;
  }

  /** Keeps `events`, each with its source's, and drops the oldest of a source past the limit. */
  add(events: readonly ObservedEvent[]): void {
    for (const event of events) {
      const source = event.by ?? "";
      const kept = this.#kept.get(source) ?? [];
      this.#order += 1;
      kept.push({ order: this.#order, event });
      if (kept.length > this.#limit) {
        kept.shift();
        this.#dropped[source] = (this.#dropped[source] ?? 0) + 1;
      }
      this.#kept.set(source, kept);
    }
    this.#merged = undefined;
  }

  get events(): readonly ObservedEvent[] {
    this.#merged ??= [...this.#kept.values()]
      .flat()
      .sort((a, b) => a.order - b.order)
      .map(({ event }) => event);
    return this.#merged;
  }

  get dropped(): Readonly<Record<FrameId, number>> {
    return { ...this.#dropped };
  }
}
