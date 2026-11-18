// The Sandbox tab's tree (docs/devtools.md, decision 7): the host's page and its frames, down to those its frames make,
// each with its level, its network, the components mounted in it with their times, and its fallbacks, from what the
// sandbox's observer kept. What a frame reported is marked with `by`, the frame whose report it is; what it says of a
// frame the host's page saw itself is kept apart, as its claim, never in place of what the host's page saw.

import type { FrameId, Gone, ObservedEvent, Renders } from "@compmd/sandbox";

type Level = Extract<ObservedEvent, { type: "frame" }>["level"];
type FrameEvent = Extract<ObservedEvent, { type: "frame" }>;
type FallbackEvent = Extract<ObservedEvent, { type: "fallback" }>;

/** A component mounted in a window. */
export interface UseNode {
  use: number;
  component: string;
  /** How long it took to mount, and its last update, in ms. */
  ms: number;
  updated?: number;
  updates: number;
  by?: FrameId;
}

/** A window of the tree: the host's page (`id` `""`), or a frame. */
export interface FrameNode {
  id: FrameId;
  level: Level | "host";
  renders?: Renders;
  network?: string[];
  /** The frame whose report this frame's own record is; none for what the host's page saw. */
  by?: FrameId;
  /** What frames reported of this frame that the host's page saw itself: shown beside it, marked. */
  claims: FrameEvent[];
  uses: UseNode[];
  fallbacks: FallbackEvent[];
  /** The last render's time it reported (`rendered`), in ms. */
  rendered?: number;
  frames: FrameNode[];
}

/** The window above `id`: `"2/1"` is in `"2"`, `"2"` in the host's page. */
export const parentOf = (id: FrameId): FrameId => id.split("/").slice(0, -1).join("/");

/** A frame's number in its parent, for the order of its siblings. */
const numberOf = (id: FrameId): number => Number(id.split("/").at(-1));

/** The tree `events` describe now: frames that went are left out, with what was in them. */
export function treeOf(events: readonly ObservedEvent[]): FrameNode {
  const nodes = new Map<FrameId, FrameNode>();
  const blank = (id: FrameId): FrameNode => ({ id, level: id === "" ? "host" : "strict", claims: [], uses: [], fallbacks: [], frames: [] });
  const node = (id: FrameId): FrameNode => {
    const found = nodes.get(id) ?? blank(id);
    nodes.set(id, found);
    return found;
  };
  const gone = new Map<FrameId, Gone>();
  for (const event of events) {
    switch (event.type) {
      case "frame": {
        const frame = node(event.frame);
        const seen = frame.renders !== undefined && frame.by === undefined;
        if (seen && event.by !== undefined) {
          frame.claims.push(event);
          break;
        }
        Object.assign(frame, { level: event.level, renders: event.renders, ...(event.network ? { network: event.network } : {}) });
        if (event.by === undefined) delete frame.by;
        else frame.by = event.by;
        break;
      }
      case "gone":
        gone.set(event.frame, event.why);
        break;
      case "mounted":
        node(event.frame).uses.push({ use: event.use, component: event.component, ms: event.ms, updates: 0, ...(event.by === undefined ? {} : { by: event.by }) });
        break;
      case "updated": {
        const use = node(event.frame).uses.find((each) => each.use === event.use);
        if (use) Object.assign(use, { updated: event.ms, updates: use.updates + 1 });
        break;
      }
      case "unmounted": {
        const frame = node(event.frame);
        frame.uses = frame.uses.filter((each) => each.use !== event.use);
        break;
      }
      case "fallback":
        node(event.frame).fallbacks.push(event);
        break;
      case "message": {
        const data = event.data as { type?: unknown; ms?: unknown };
        if (event.way === "up" && data.type === "rendered" && typeof data.ms === "number") node(event.frame).rendered = data.ms;
        break;
      }
      default:
        break;
    }
  }
  const root = node("");
  const live = [...nodes.values()].filter(({ id }) => id !== "" && ![...gone.keys()].some((went) => id === went || id.startsWith(`${went}/`)));
  for (const frame of live.sort((a, b) => a.id.split("/").length - b.id.split("/").length || numberOf(a.id) - numberOf(b.id))) {
    (nodes.get(parentOf(frame.id)) ?? root).frames.push(frame);
  }
  return root;
}
