// How the observer tabs name frames and times, as text only (docs/devtools.md, O1).

import type { FrameId, Renders } from "@compmd/sandbox";

/** A frame by its id: the host's page, or `frame 3/1`. */
export const frameName = (id: FrameId): string => (id === "" ? "the host's page" : `frame ${id}`);

/** What a frame renders, in a few words. */
export function rendersText(renders: Renders | undefined): string {
  if (!renders) return "";
  if (renders.kind === "page") return "the page";
  if (renders.kind === "compile") return "the compiles";
  return renders.component;
}

/** A time in milliseconds since the epoch as the clock shows it, to the millisecond: `14:03:07.218`. */
export function clockOf(at: number): string {
  const date = new Date(at);
  const two = (value: number) => String(value).padStart(2, "0");
  return `${two(date.getHours())}:${two(date.getMinutes())}:${two(date.getSeconds())}.${String(date.getMilliseconds()).padStart(3, "0")}`;
}

/** A duration in ms, as short as it can be read: `0.25 ms`, `12.4 ms`, `310 ms`. */
export function msText(ms: number): string {
  return `${ms < 1 ? ms.toFixed(2) : ms < 100 ? ms.toFixed(1) : ms.toFixed(0)} ms`;
}
