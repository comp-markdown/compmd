// What the browser tests share: the servers `setup.ts` started, and a probe component in a frame of its own, which
// runs a test's code there and sets an input to what it found.

import type { ExpandedComponent } from "@compmd/render";
import { inject } from "vitest";
import type { ComponentCode, Level } from "../src/protocol/messages.ts";
import { frameUse, type UseFrame, type UseFrameOptions } from "../src/use.ts";

export const frames = inject("frames");
export const allowed = inject("allowed");
export const denied = inject("denied");

/**
 * A tag for one test's requests. Chrome and WebKit run at once against the same servers, so a test counts only what
 * carries its tag.
 */
export const tagged = () => `t${crypto.randomUUID().replaceAll("-", "")}`;

/** The requests that reached `server` with `tag` in them, each as `<path>?<query>`. */
export async function hits(server: string, tag: string): Promise<string[]> {
  return ((await (await fetch(`${server}hits`)).json()) as string[]).filter((hit) => hit.includes(tag));
}

/** Waits `ms`, for requests that would have been made to arrive. */
export const pause = (ms: number) =>
  new Promise((resolve) => {
    setTimeout(resolve, ms);
  });

/** A leaf use of `name` whose `result` param is bound to the input `result`. */
export const leaf = (name: string): ExpandedComponent => ({
  type: "componentLeaf",
  name,
  params: [{ key: "result", breakpoint: null, value: "", input: "result" }],
  fallback: [{ type: "text", value: `${name}'s fallback` }],
});

/**
 * A plain component whose mount runs `body`, an async function of `(target, context)` written as text, in its frame,
 * and sets the input `result` to the JSON of what it returns.
 */
export function probeCode(body: string): string {
  return `export default function mount(target, context) {
  (${body})(target, context).then(
    (result) => context.set("result", JSON.stringify(result ?? null)),
    (error) => context.set("result", JSON.stringify({ crashed: String(error) })),
  );
}`;
}

export interface Probe {
  frame: UseFrame;
  /** What the probe's code returned. */
  result: Promise<unknown>;
}

/** Runs `body` in a frame of `level`, as the component `probe`, in `target` (the test page's body by default). */
export function probe(level: Level, body: string, options: Partial<UseFrameOptions> & { target?: Element; extra?: ComponentCode[] } = {}): Probe {
  let resolve: (value: unknown) => void = () => undefined;
  const result = new Promise((done) => (resolve = done));
  const frame = frameUse(options.target ?? document.body, {
    url: `${frames}${level}.html`,
    level,
    code: [{ name: "probe", level, binding: "plain", code: probeCode(body) }, ...(options.extra ?? [])],
    node: leaf("probe"),
    inputs: { result: { type: "string", value: "" } },
    ...options,
    on: {
      set: ({ input, value }) => {
        if (input === "result") resolve(JSON.parse(value as string));
      },
      ...options.on,
    },
  });
  return { frame, result };
}

/** A test's code, run in a frame: tries `attempt` and says whether it reached its target or was refused. */
export const ATTEMPT = `const attempt = (run) => new Promise((resolve) => {
  try { Promise.resolve(run()).then(() => resolve("reached"), () => resolve("refused")); } catch { resolve("refused"); }
});
const loads = (element) => new Promise((resolve, reject) => { element.onload = resolve; element.onerror = reject; document.head.append(element); });`;
