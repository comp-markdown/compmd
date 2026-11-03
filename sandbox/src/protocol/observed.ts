// What a window's observer records (docs/devtools.md, decision 6), as the protocol carries it up in `observed`: the
// events, a check of each one, which code of a lower level may have written, and the ids that nest a frame's events
// under it, so a frame can only report on its own part of the tree.

import type { ErrorReason, Level } from "./messages.ts";

/**
 * A frame, by its path from the window that records it: `""` is that window itself, `"2"` its second frame, `"2/1"`
 * that frame's first. A parent puts its child's id in front of every id the child reports.
 */
export type FrameId = string;

/** What a frame renders: the page (frame mode), the compile frame's compiles, or one use of a component. */
export type Renders = { kind: "page" } | { kind: "compile" } | { kind: "use"; component: string };

/** Why a frame went: removed with its use or view, stopped when it hung, or of another protocol version. */
export type Gone = "removed" | "hung" | "version";

/** One thing a window's observer saw, at `at`, in milliseconds since the epoch, as windows share them. */
export type Observed =
  | { type: "frame"; frame: FrameId; level: Level; renders: Renders; network?: string[]; at: number }
  | { type: "gone"; frame: FrameId; why: Gone; at: number }
  /** A message between the window and its frame `frame`: `down` to it, `up` from it; large values kept by size. */
  | { type: "message"; frame: FrameId; way: "down" | "up"; data: unknown; at: number }
  /** A component mounted in the window `frame`, as its use `use`, which took `ms`. */
  | { type: "mounted"; frame: FrameId; use: number; component: string; ms: number; at: number }
  | { type: "updated"; frame: FrameId; use: number; ms: number; at: number }
  | { type: "unmounted"; frame: FrameId; use: number; at: number }
  /** A component shows its fallback in the window `frame`. */
  | { type: "fallback"; frame: FrameId; component: string; reason: ErrorReason; message: string; at: number }
  /** The window `frame` dropped `count` events it couldn't keep. */
  | { type: "dropped"; frame: FrameId; count: number; at: number };

/** The time `at` of an event recorded now. */
export const now = (): number => performance.timeOrigin + performance.now();

/** The id `frame` has in the window above the child `child`, which reported it. */
export const within = (child: FrameId, frame: FrameId): FrameId => (frame === "" ? child : `${child}/${frame}`);

/** How much of a value a copy keeps: longer strings and lists, deeper values and more of them are kept by size. */
const LIMITS = { text: 2000, items: 200, depth: 16, values: 5000 };

/**
 * A copy of `value` small enough to keep: a string longer than 2,000 characters becomes `{ $text: length, start }`, a
 * list longer than 200 items keeps its first 200 and `{ $more: count }`, a `Blob` becomes `{ $blob: size, mime }`, bytes
 * `{ $bytes: length }`, a port `{ $port: true }`, and a value too deep or past 5,000 values `{ $more: 1 }`.
 *
 * @example
 * copyOf({ type: "compiled", code: longText }); // { type: "compiled", code: { $text: 48210, start: "import …" } }
 */
export function copyOf(value: unknown): unknown {
  let values = 0;
  const copy = (item: unknown, depth: number): unknown => {
    values += 1;
    if (values > LIMITS.values || depth > LIMITS.depth) return { $more: 1 };
    if (typeof item === "string") return item.length > LIMITS.text ? { $text: item.length, start: item.slice(0, LIMITS.text) } : item;
    if (item === null || typeof item !== "object") return typeof item === "function" || typeof item === "symbol" ? String(item) : item;
    if (typeof Blob !== "undefined" && item instanceof Blob) return { $blob: item.size, mime: item.type };
    if (ArrayBuffer.isView(item) || item instanceof ArrayBuffer) return { $bytes: item.byteLength };
    if (typeof MessagePort !== "undefined" && item instanceof MessagePort) return { $port: true };
    if (Array.isArray(item)) {
      const kept = item.slice(0, LIMITS.items).map((entry: unknown) => copy(entry, depth + 1));
      return item.length > LIMITS.items ? [...kept, { $more: item.length - LIMITS.items }] : kept;
    }
    const entries = Object.entries(item);
    const kept = Object.fromEntries(entries.slice(0, LIMITS.items).map(([key, entry]) => [key, copy(entry, depth + 1)]));
    return entries.length > LIMITS.items ? { ...kept, $more: entries.length - LIMITS.items } : kept;
  };
  return copy(value, 0);
}

const LEVELS: readonly string[] = ["strict", "permissive"];
const REASONS: readonly string[] = ["missing", "threw", "failed", "not-here"];
const GONE: readonly string[] = ["removed", "hung", "version"];

const isText = (value: unknown, max = 1000): value is string => typeof value === "string" && value.length <= max;
const isCount = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value) && value >= 0;
const isFrameId = (value: unknown): value is FrameId => isText(value, 200) && /^(\d+(\/\d+)*)?$/.test(value);

function rendersOf(value: unknown): Renders | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const { kind, component } = value as { kind?: unknown; component?: unknown };
  if (kind === "page" || kind === "compile") return { kind };
  return kind === "use" && isText(component) ? { kind, component } : undefined;
}

/**
 * `data` as an event a frame may report, rebuilt from its fields, with large values kept by size; `undefined` when it
 * isn't one. A frame's report comes from code of its level, so nothing in it is taken as it is.
 */
export function observedOf(data: unknown): Observed | undefined {
  if (typeof data !== "object" || data === null) return undefined;
  const event = data as Record<string, unknown>;
  const { frame, at } = event;
  if (!isFrameId(frame) || !isCount(at)) return undefined;
  switch (event.type) {
    case "frame": {
      const renders = rendersOf(event.renders);
      const { level, network } = event;
      if (!isText(level) || !LEVELS.includes(level) || !renders) return undefined;
      const origins = Array.isArray(network) ? network.slice(0, LIMITS.items).filter((origin) => isText(origin)) : undefined;
      return { type: "frame", frame, level: level as Level, renders, ...(origins ? { network: origins } : {}), at };
    }
    case "gone":
      return isText(event.why) && GONE.includes(event.why) ? { type: "gone", frame, why: event.why as Gone, at } : undefined;
    case "message":
      return event.way === "down" || event.way === "up" ? { type: "message", frame, way: event.way, data: copyOf(event.data), at } : undefined;
    case "mounted":
      return isCount(event.use) && isText(event.component) && isCount(event.ms) ? { type: "mounted", frame, use: event.use, component: event.component, ms: event.ms, at } : undefined;
    case "updated":
      return isCount(event.use) && isCount(event.ms) ? { type: "updated", frame, use: event.use, ms: event.ms, at } : undefined;
    case "unmounted":
      return isCount(event.use) ? { type: "unmounted", frame, use: event.use, at } : undefined;
    case "fallback": {
      const { component, reason, message } = event;
      if (!isText(component) || !isText(reason) || !REASONS.includes(reason) || typeof message !== "string") return undefined;
      return { type: "fallback", frame, component, reason: reason as ErrorReason, message: message.length > LIMITS.text ? `${message.slice(0, LIMITS.text)}…` : message, at };
    }
    case "dropped":
      return isCount(event.count) ? { type: "dropped", frame, count: event.count, at } : undefined;
    default:
      return undefined;
  }
}
