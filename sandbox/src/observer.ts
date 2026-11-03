// The observer of one window (docs/devtools.md, decisions 5 and 6), the host's page or a frame: it records what passes
// in the window, the frames it makes, the messages on their ports, the components it mounts and the fallbacks they
// show, and relays what its frames report under their ids. It records nothing until it is started: the host's page
// starts its own, and a frame's starts when its parent asks. What it keeps while stopped is only what is live, the
// frames and mounted components, so that a start reports the tree as it is.

import type { ErrorReason, Level, Port } from "./protocol/messages.ts";
import { copyOf, now, observedOf, within, type FrameId, type Gone, type Observed, type Renders } from "./protocol/observed.ts";
import type { ChildEnd } from "./protocol/parent-end.ts";

/** Where an observer's events go, with `by`, the frame whose report they are, or none for what the window saw. */
export type Sink = (events: Observed[], by?: FrameId) => void;

/**
 * Messages an observer doesn't copy: its own, and pings and pongs, which come once a second from every frame and would
 * push everything else out of what is kept.
 */
const UNCOPIED: readonly unknown[] = ["observe", "observed", "ping", "pong"];

const isUncopied = (data: unknown): boolean => typeof data === "object" && data !== null && UNCOPIED.includes((data as { type?: unknown }).type);

type FrameEvent = Extract<Observed, { type: "frame" }>;
type MountedEvent = Extract<Observed, { type: "mounted" }>;

/**
 * What one window observes.
 *
 * @example
 * const observer = new Observer((events) => log.add(events));
 * const id = observer.frame("strict", { kind: "use", component: "chart" });
 * const child = connectChild(observer.port(id, port), options);
 * observer.connected(id, child);
 * observer.start();
 */
export class Observer {
  readonly #sink: Sink;
  #on = false;
  #frames = 0;
  #uses = 0;
  readonly #live = new Map<FrameId, FrameEvent>();
  readonly #mounted = new Map<number, MountedEvent>();
  readonly #children = new Map<FrameId, ChildEnd>();

  constructor(sink: Sink) {
    this.#sink = sink;
  }

  get on(): boolean {
    return this.#on;
  }

  /** The frames the window made and the components mounted in it, as their events, that are live now. */
  live(): Observed[] {
    return [...this.#live.values(), ...this.#mounted.values()];
  }

  /** Starts recording: reports the live frames and components, and asks every frame to report its own part. */
  start(): void {
    if (this.#on) return;
    this.#on = true;
    this.#sink(this.live());
    for (const child of this.#children.values()) child.observe(true);
  }

  /** Stops recording, and asks every frame to stop. */
  stop(): void {
    if (!this.#on) return;
    this.#on = false;
    for (const child of this.#children.values()) child.observe(false);
  }

  /** The window made a frame of `level` for `renders`: its id. */
  frame(level: Level, renders: Renders, network?: readonly string[]): FrameId {
    this.#frames += 1;
    const frame = String(this.#frames);
    const event: FrameEvent = { type: "frame", frame, level, renders, ...(network ? { network: [...network] } : {}), at: now() };
    this.#live.set(frame, event);
    this.#record(event);
    return frame;
  }

  /** The frame `frame` went, for `why`; once, however often it is said. */
  gone(frame: FrameId, why: Gone): void {
    if (!this.#live.delete(frame)) return;
    this.#children.delete(frame);
    this.#record({ type: "gone", frame, why, at: now() });
  }

  /** `port`, the window's end of the frame `frame`'s port, with every message on it copied while recording. */
  port(frame: FrameId, port: Port): Port {
    port.addEventListener("message", ({ data }) => {
      this.#copy(frame, "up", data);
    });
    return {
      postMessage: (message, transfer) => {
        this.#copy(frame, "down", message);
        if (transfer) port.postMessage(message, transfer);
        else port.postMessage(message);
      },
      addEventListener: (type, listener) => {
        port.addEventListener(type, listener);
      },
      removeEventListener: (type, listener) => {
        port.removeEventListener(type, listener);
      },
      start: () => port.start?.(),
    };
  }

  /**
   * The frame `frame` is connected on `child`: while recording, it is asked to report, and its reports go on with its
   * id in front of each of theirs, so it can only report on frames under it.
   */
  connected(frame: FrameId, child: ChildEnd): void {
    this.#children.set(frame, child);
    child.on("observed", ({ events }) => {
      if (!this.#on || this.#children.get(frame) !== child) return;
      const checked = events.map(observedOf).filter((event) => event !== undefined);
      if (checked.length > 0) this.#sink(checked.map((event) => ({ ...event, frame: within(frame, event.frame) })), frame);
    });
    if (this.#on) child.observe(true);
  }

  /** A component mounted in the window, in `ms`: its use's number, for `updated` and `unmounted`. */
  mounted(component: string, ms: number): number {
    this.#uses += 1;
    const event: MountedEvent = { type: "mounted", frame: "", use: this.#uses, component, ms, at: now() };
    this.#mounted.set(this.#uses, event);
    this.#record(event);
    return this.#uses;
  }

  updated(use: number, ms: number): void {
    this.#record({ type: "updated", frame: "", use, ms, at: now() });
  }

  unmounted(use: number): void {
    if (!this.#mounted.delete(use)) return;
    this.#record({ type: "unmounted", frame: "", use, at: now() });
  }

  /** A component shows its fallback in the window. */
  fallback(component: string, reason: ErrorReason, message: string): void {
    this.#record({ type: "fallback", frame: "", component, reason, message, at: now() });
  }

  #record(event: Observed): void {
    if (this.#on) this.#sink([event]);
  }

  #copy(frame: FrameId, way: "down" | "up", data: unknown): void {
    if (!this.#on || isUncopied(data)) return;
    this.#record({ type: "message", frame, way, data: copyOf(data), at: now() });
  }
}

/** The most events a frame holds between two reports; past it, it counts what it drops. */
const HELD = 2000;

/**
 * A frame's sink: its events in batches, sent with `send` every `every` ms, at most 2,000 of them, with a `dropped`
 * event for the rest; `clear` drops what waits, when the parent asks the frame to stop.
 */
export function batches(send: (events: Observed[]) => void, every = 100): { sink: Sink; clear: () => void } {
  let held: Observed[] = [];
  let dropped = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const flush = () => {
    timer = undefined;
    const batch = dropped > 0 ? [...held, { type: "dropped" as const, frame: "", count: dropped, at: now() }] : held;
    held = [];
    dropped = 0;
    send(batch);
  };
  return {
    sink: (events) => {
      const room = Math.max(0, HELD - held.length);
      held.push(...events.slice(0, room));
      dropped += Math.max(0, events.length - room);
      timer ??= setTimeout(flush, every);
    },
    clear: () => {
      if (timer !== undefined) clearTimeout(timer);
      timer = undefined;
      held = [];
      dropped = 0;
    },
  };
}
