// The self-check (docs/tauri-plugin.md, decision 9): what the page drew in a real webview, after it has settled. The
// strict frames' insides are another origin's, which the page can't read, so the sandbox's observer says what mounted
// in them, and the component sets an input to its frame's origin.

import type { Observation, View, ViewEvents } from "@compmd/sandbox";

/** How the app runs: showing the document, checking it, or checking that broken frames say so. */
export type Mode = "show" | "check" | "check-broken";

/** What the page drew, and saw. */
export interface Drawn {
  view: View;
  container: HTMLElement;
  observation: Observation;
  errors: readonly ViewEvents["error"][];
  sets: readonly ViewEvents["set"][];
  diagnostics: readonly unknown[];
}

/** The check's outcome: whether it passed, and what it found. */
export interface Report {
  ok: boolean;
  message: string;
}

/** How long a check waits for the page to settle and say what it needs, in ms. */
const WAIT_MS = 30_000;

/** Waits until `done` holds, or `WAIT_MS` has passed. */
async function until(done: () => boolean): Promise<void> {
  const end = Date.now() + WAIT_MS;
  while (!done() && Date.now() < end) await new Promise((resolve) => setTimeout(resolve, 50));
}

/** Whether a component named `name` mounted in one of the page's frames. */
const mounted = (observation: Observation, name: string) =>
  observation.events.some((event) => event.type === "mounted" && event.component === name && event.frame !== "");

/** What the component reported its frame sees, from the value of its `set`. */
function seenBy(value: unknown): { origin?: unknown; tauri?: unknown } | undefined {
  try {
    return JSON.parse(String(value)) as { origin?: unknown; tauri?: unknown };
  } catch {
    return undefined;
  }
}

/** Checks what `drawn` shows, as `mode` expects it. */
export async function check(mode: Exclude<Mode, "show">, drawn: Drawn): Promise<Report> {
  const failed: string[] = [];
  const { observation, errors, sets, container } = drawn;
  if (mode === "check-broken") {
    await until(() => errors.length > 0);
    if (!errors.some(({ reason, message }) => reason === "failed" && message.includes("didn't load its runtime"))) failed.push(`no error for frames whose runtime is missing (errors: ${JSON.stringify(errors)})`);
    return failed.length === 0 ? { ok: true, message: "broken frames: the sandbox said so" } : { ok: false, message: failed.join("; ") };
  }
  // Waiting for the page to settle is a courtesy, not a check: `settled` waits for an animation frame, which WebKit may
  // hold back for a window that isn't on screen. Each check below waits for what it looks at.
  await Promise.race([drawn.view.settled(), new Promise((resolve) => setTimeout(resolve, 5000))]);
  const highlighted = () => (container.querySelector("pre code.language-rust")?.querySelectorAll("span").length ?? 0) > 1;
  const origin = () => sets.find(({ input }) => input === "origin");
  await until(() => (mounted(observation, "mermaid") && mounted(observation, "probe") && highlighted() && origin() !== undefined) || errors.length > 0);
  if (drawn.diagnostics.length > 0) failed.push(`diagnostics: ${JSON.stringify(drawn.diagnostics)}`);
  if (errors.length > 0) failed.push(`errors: ${JSON.stringify(errors)}`);
  if (!mounted(observation, "mermaid")) failed.push("the Mermaid diagram didn't mount in its frame");
  if (!mounted(observation, "probe")) failed.push("the JSX component didn't mount in its frame");
  if (!highlighted()) failed.push("the code block isn't highlighted");
  // The component set its input to what its frame sees, as JSON text.
  const seen = seenBy(origin()?.value);
  if (seen?.origin !== "null") failed.push(`the component's frame isn't sandboxed: it sees ${JSON.stringify(seen)}`);
  if (seen?.tauri !== false) failed.push(`the component's frame can reach Tauri: it sees ${JSON.stringify(seen)}`);
  const fallbacks = observation.events.filter((event) => event.type === "fallback");
  if (fallbacks.length > 0) failed.push(`fallbacks: ${JSON.stringify(fallbacks)}`);
  return failed.length === 0 ? { ok: true, message: "the diagram, the highlighted code and the component in its strict frame drew" } : { ok: false, message: failed.join("; ") };
}
