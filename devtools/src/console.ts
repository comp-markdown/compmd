// The Console tab (docs/devtools.md, decision 7): a history of what went wrong, newest last. The diagnostics as each
// change left them, and what the editor couldn't do (a pack it couldn't fetch, a request the worker couldn't open), come
// from the editor; component fallbacks with why, refused `set`s and frames stopped come from the sandbox's observer.

import type { Diagnostic } from "@compmd/processor";
import { diagnosticOf, type FrameId, type ObservedEvent, type SandboxDiagnostic } from "@compmd/sandbox";

/** One line of the console, at `at`, in milliseconds since the epoch. */
export type ConsoleEntry =
  | { kind: "diagnostics"; at: number; diagnostics: readonly Diagnostic[] }
  | { kind: "problem"; at: number; message: string }
  | { kind: "fallback"; at: number; frame: FrameId; component: string; reason: string; code?: SandboxDiagnostic; message: string; by?: FrameId }
  | { kind: "refused"; at: number; frame: FrameId; input: string; by?: FrameId }
  | { kind: "stopped"; at: number; frame: FrameId; why: "hung" | "version"; by?: FrameId };

/** The time of something that happens now, as the observer's events have it. */
export const now = (): number => performance.timeOrigin + performance.now();

/** A diagnostic as one line, to tell one change's from the next. */
const keyOf = (diagnostics: readonly Diagnostic[]): string => diagnostics.map((d) => `${d.code} ${d.file ?? ""} ${String(d.range.start.line)}:${String(d.range.start.column)} ${d.message}`).join("\n");

/**
 * What the editor said went wrong, as it says it: each change's diagnostics when they differ from the last change's, and
 * each problem once, when it is first listed.
 */
export class EditorHistory {
  readonly entries: ConsoleEntry[] = [];
  #diagnostics = "";
  #problems = new Set<string>();

  /** A change left `diagnostics` and `problems`, at `at`. */
  note(diagnostics: readonly Diagnostic[], problems: readonly string[], at = now()): void {
    const key = keyOf(diagnostics);
    if (key !== this.#diagnostics) this.entries.push({ kind: "diagnostics", at, diagnostics });
    this.#diagnostics = key;
    for (const message of problems) if (!this.#problems.has(message)) this.entries.push({ kind: "problem", at, message });
    this.#problems = new Set(problems);
  }
}

const marked = (by: FrameId | undefined) => (by === undefined ? {} : { by });

/** What went wrong in the sandbox, of what its observer kept. */
function sandboxEntries(events: readonly ObservedEvent[]): ConsoleEntry[] {
  return events.flatMap((event): ConsoleEntry[] => {
    const { at, frame, by } = event;
    switch (event.type) {
      case "fallback": {
        const code = diagnosticOf(event.reason);
        return [{ kind: "fallback", at, frame, component: event.component, reason: event.reason, ...(code ? { code } : {}), message: event.message, ...marked(by) }];
      }
      case "refused":
        return [{ kind: "refused", at, frame, input: event.input, ...marked(by) }];
      case "gone":
        return event.why === "removed" ? [] : [{ kind: "stopped", at, frame, why: event.why, ...marked(by) }];
      default:
        return [];
    }
  });
}

/** The console's lines, oldest first: the editor's and the sandbox's, in the order they happened. */
export function consoleOf(editor: readonly ConsoleEntry[], events: readonly ObservedEvent[]): ConsoleEntry[] {
  return [...editor, ...sandboxEntries(events)].sort((a, b) => a.at - b.at);
}
