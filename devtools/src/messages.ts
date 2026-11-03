// The Messages tab (docs/devtools.md, decision 7): the sandbox's protocol on a timeline, a lane per frame, each message
// with its direction, its type and its copy, as the sandbox's observer kept it.

import type { FrameId, ObservedEvent } from "@compmd/sandbox";

/** One message on the timeline. */
export interface MessageRow {
  /** Its place in what was kept, which keys it. */
  index: number;
  at: number;
  /** The frame it went to or came from, whose lane it is in. */
  frame: FrameId;
  way: "down" | "up";
  /** Its `type`, as text; `?` for a message without one. */
  type: string;
  data: unknown;
  by?: FrameId;
}

export interface Timeline {
  /** The frames with messages, in the tree's order: each one's lane. */
  lanes: FrameId[];
  rows: MessageRow[];
}

/** The order of frames in the tree: `1` before `1/1` before `2`. */
const byPath = (a: FrameId, b: FrameId): number => {
  const [left, right] = [a.split("/").map(Number), b.split("/").map(Number)];
  for (let at = 0; at < Math.min(left.length, right.length); at++) if (left[at] !== right[at]) return (left[at] ?? 0) - (right[at] ?? 0);
  return left.length - right.length;
};

/** The timeline of the messages in `events`. */
export function timelineOf(events: readonly ObservedEvent[]): Timeline {
  const rows = events.flatMap((event, index): MessageRow[] => {
    if (event.type !== "message") return [];
    const type = (event.data as { type?: unknown } | null)?.type;
    return [{ index, at: event.at, frame: event.frame, way: event.way, type: typeof type === "string" ? type : "?", data: event.data, ...(event.by === undefined ? {} : { by: event.by }) }];
  });
  return { lanes: [...new Set(rows.map(({ frame }) => frame))].sort(byPath), rows };
}
