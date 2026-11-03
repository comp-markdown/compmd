// The Console tab (docs/devtools.md, decision 7): a history of what went wrong, newest last. The diagnostics as each
// change left them, and what the editor couldn't do (a pack it couldn't fetch, a request the worker couldn't open), come
// from the editor; component fallbacks with why, refused `set`s and frames stopped come from the sandbox's observer.
// Among them, the expressions the developer evaluated in the document, with their values.

import type { Diagnostic, Evaluated } from "@compmd/processor";
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

/**
 * An expression the developer evaluated in the document, at `at`: its value, its F024, why it couldn't be evaluated
 * (`failed`, a document the processor can't open), or none yet while the worker works.
 */
export interface Evaluation {
  id: number;
  at: number;
  expression: string;
  result?: Evaluated | { failed: string };
}

/** The console's lines and the evaluations, oldest first, as one list. */
export function withEvaluations(entries: readonly ConsoleEntry[], evaluations: readonly Evaluation[]): (ConsoleEntry | ({ kind: "evaluation" } & Evaluation))[] {
  return [...entries, ...evaluations.map((evaluation) => ({ kind: "evaluation" as const, ...evaluation }))].sort((a, b) => a.at - b.at);
}

/**
 * `expression` with a mark, `^`, under the code point at `line` and `column` (1-based), on a line of its own after that
 * line: where an F024 is.
 *
 * @example
 * markedAt("md.doc.tem", 1, 8); // "md.doc.tem\n       ^"
 */
export function markedAt(expression: string, line: number, column: number): string {
  const lines = expression.split("\n");
  const mark = `${" ".repeat(Math.max(0, column - 1))}^`;
  return [...lines.slice(0, line), mark, ...lines.slice(line)].join("\n");
}

/**
 * The expressions evaluated, oldest first, for ↑ and ↓ as a browser's console walks them: ↑ gives the one before the
 * one shown, ↓ the one after, and past the newest, what was being written.
 */
export class ExpressionHistory {
  readonly #expressions: string[] = [];
  /** Where ↑ and ↓ are: an index into the expressions, or their length for what is being written. */
  #at = 0;
  #draft = "";

  /** `expression` was evaluated: it is the newest, and the walk starts again after it. */
  add(expression: string): void {
    if (this.#expressions.at(-1) !== expression) this.#expressions.push(expression);
    this.#at = this.#expressions.length;
    this.#draft = "";
  }

  /** The expression before the one shown, `current`, which is kept when it is what was being written; or none. */
  previous(current: string): string | undefined {
    if (this.#at === 0) return undefined;
    if (this.#at === this.#expressions.length) this.#draft = current;
    this.#at -= 1;
    return this.#expressions[this.#at];
  }

  /** The expression after the one shown, or what was being written past the newest; or none at the end. */
  next(): string | undefined {
    if (this.#at >= this.#expressions.length) return undefined;
    this.#at += 1;
    return this.#at === this.#expressions.length ? this.#draft : this.#expressions[this.#at];
  }
}
