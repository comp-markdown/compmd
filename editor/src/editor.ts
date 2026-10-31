// The editor's loop: each input in the text area becomes an edit, the worker processes it, and the page's sandbox
// applies the splices it gave, which the editor passes on unread. An input component's `set` on the page, or the inputs
// panel, opens the document again with the new value. It times each change from its first key press to the page painted.

import type { Diagnostic, ExpandedInput, PositionalHint, Request } from "@compmd/processor";
import type { Change } from "@compmd/render";
import type { View } from "@compmd/sandbox";
import { editOf, fixOf, type Selection } from "./edit.ts";
import type { Example } from "./documents.ts";
import { Session, type Hinted, type Update, type WorkerPort } from "./session.ts";
import { Timings } from "./timings.ts";

/** What the editor shows besides the page. */
export interface EditorState {
  diagnostics: Diagnostic[];
  /** The document's inputs, by name, with the values the page shows. */
  inputs: Record<string, ExpandedInput>;
  /** Components the frame couldn't compile, load or run, and why. */
  problems: string[];
  /** The worker's processing, per change. */
  processing: Timings;
  /** The frame's part, per change: from the splices' arrival to the frame painted, as the frame reports it. */
  rendering: Timings;
  /** From a change's first key press (or opening) to the frame painted. */
  painted: Timings;
  /** How many times the worker's answers have changed the document, for views that follow it. */
  updates: number;
  /** What is at the caret, in the text the worker processed last. */
  hint?: PositionalHint;
}

/** The text after a fix or a completion, for the text area, and where its caret goes: past what was written. */
export interface Replaced {
  text: string;
  caret: number;
}

/** What the editor uses of the document's view in its sandbox: its splices, its components and what it reports. */
export type Frame = Pick<View, "splice" | "components" | "on">;

/** A processor diagnostic as the renderer's rules for invalid text read it: the processed document's only. */
const forRenderer = (diagnostics: Diagnostic[]): NonNullable<Change["diagnostics"]> =>
  diagnostics.filter((d) => d.file === undefined).map((d) => ({ code: d.code, line: d.range.start.line, column: d.range.start.column }));

export class Editor {
  readonly state: EditorState = { diagnostics: [], inputs: {}, problems: [], processing: new Timings(), rendering: new Timings(), painted: new Timings(), updates: 0 };
  readonly #session: Session;
  readonly #frame: Frame;
  readonly #onChange: () => void;
  /** The text the last input left, and the selection before the next one. */
  #text = "";
  /** The text the diagnostics shown are of, and the one the hint is of. */
  #processed: string | undefined;
  #hinted: string | undefined;
  #selection: Selection = { start: 0, end: 0 };
  #request: Request | undefined;
  #inputs: Record<string, unknown> = {};
  /** When each render sent to the frame started, for its paint. */
  readonly #rendering: number[] = [];

  /** Runs with `worker` and `frame`; `onChange` is called when the state changes. */
  constructor(worker: WorkerPort, frame: Frame, onChange: () => void) {
    this.#frame = frame;
    this.#onChange = onChange;
    this.#session = new Session(
      worker,
      (update) => {
        this.#updated(update);
      },
      (message) => {
        this.state.problems = [message];
        onChange();
      },
      (hinted) => {
        this.#hint(hinted);
        onChange();
      },
    );
    frame.on("rendered", ({ ms }) => {
      const since = this.#rendering.shift();
      if (since === undefined) return;
      this.state.painted.add(performance.now() - since);
      this.state.rendering.add(ms);
      onChange();
    });
    frame.on("set", ({ input, value }) => {
      this.setInput(input, value);
    });
    // A component that couldn't compile, load or run, or isn't allowed where it is used; a page's own problem has none.
    frame.on("error", ({ component, message }) => {
      if (component) this.#problem(`${component}: ${message}`);
    });
  }

  /** Opens `example`: its text is the text area's. Its nodes carry origins, which the renderer's keys come from. */
  open(example: Example, at = performance.now()): string {
    this.#request = { ...example.request, origins: true };
    this.#inputs = {};
    this.#text = example.request.files?.[example.request.path] ?? "";
    this.state.problems = [];
    delete this.state.hint;
    this.#session.open(this.#request, this.#text, at);
    return this.#text;
  }

  /** Before an input: what is selected, which the input replaces. */
  beforeInput(selection: Selection): void {
    this.#selection = selection;
  }

  /** After an input, at `at`, the text area holds `text`, with its caret at `caret`. */
  input(text: string, caret: number, at = performance.now()): void {
    const edit = editOf(this.#text, text, this.#selection);
    this.#text = text;
    this.#session.edit(edit, text, caret, at);
  }

  /** The caret moved to `caret` without an input: the hint follows it. */
  moveCaret(caret: number): void {
    this.#session.moveCaret(caret);
  }

  /**
   * Makes a diagnostic's fix, as an input would, and gives the text after it for the text area. A fix is of the text
   * its diagnostic was found in, so after another input it does nothing (`undefined`) until the diagnostics come
   * again.
   */
  fix(fix: NonNullable<Diagnostic["fix"]>, at = performance.now()): Replaced | undefined {
    if (this.#text !== this.#processed) return undefined;
    return this.#replace(fix, at);
  }

  /** Writes `text` in place of what the hint is about, as a fix does: only in the text the hint is of. */
  complete(text: string, at = performance.now()): Replaced | undefined {
    const hint = this.state.hint;
    if (!hint || this.#text !== this.#hinted) return undefined;
    return this.#replace({ range: hint.range, text }, at);
  }

  #replace(fix: NonNullable<Diagnostic["fix"]>, at: number): Replaced {
    const edit = fixOf(this.#text, fix);
    this.#text = this.#text.slice(0, edit.start) + edit.text + this.#text.slice(edit.end);
    const caret = edit.start + edit.text.length;
    this.#session.edit(edit, this.#text, caret, at);
    return { text: this.#text, caret };
  }

  /** Gives the input `name` the value `value`, as the page's input components do: the document opens again with it. */
  setInput(name: string, value: unknown): void {
    this.#inputs = { ...this.#inputs, [name]: value };
    this.#reopen();
  }

  /** The document as it is now, with the current text and the inputs' values: a request to process it from scratch. */
  get document(): Request | undefined {
    const request = this.#request;
    if (!request) return undefined;
    const files = { ...request.files, [request.path]: this.#text };
    return { ...request, files, inputs: { ...request.inputs, ...this.#inputs } };
  }

  /** The document again, with the current text and the inputs' values. */
  #reopen(): void {
    const document = this.document;
    if (document) this.#session.open(document, this.#text);
  }

  #updated(update: Update): void {
    if (update.components) void this.#frame.components(update.components);
    const change: Change = { text: update.text, diagnostics: forRenderer(update.diagnostics) };
    this.#rendering.push(update.since);
    this.#frame.splice(update.splices, change, { replace: update.opened, inputs: update.inputs });
    this.state.diagnostics = update.diagnostics;
    this.state.inputs = update.inputs;
    this.#processed = update.text;
    this.#hint(update);
    this.state.updates++;
    this.state.processing.add(update.ms);
    if (update.components) this.state.problems = [];
    this.#onChange();
  }

  #hint({ hint, text }: Hinted): void {
    this.#hinted = text;
    if (hint) this.state.hint = hint;
    else delete this.state.hint;
  }

  #problem(problem: string): void {
    if (!this.state.problems.includes(problem)) this.state.problems = [...this.state.problems, problem];
    this.#onChange();
  }
}
