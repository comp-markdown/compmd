// The editor's loop: each input in the text area becomes an edit, the worker processes it, and the frame applies the
// splices it gave, which the editor passes on unread. An input component's `set` in the frame opens the document
// again with the new value. It times each change from its first key press to the frame painted.

import type { Diagnostic, Request } from "@compmd/processor";
import type { Change } from "@compmd/render";
import type { FrameConnection } from "@compmd/render/frame";
import { editOf, fixOf, type Selection } from "./edit.ts";
import type { Example } from "./documents.ts";
import type { ComponentFailed, ComponentsMessage } from "./protocol.ts";
import { Session, type Update, type WorkerPort } from "./session.ts";
import { Timings } from "./timings.ts";

/** What the editor shows besides the page. */
export interface EditorState {
  diagnostics: Diagnostic[];
  /** Components the frame couldn't compile, load or run, and why. */
  problems: string[];
  /** The worker's processing, per change. */
  processing: Timings;
  /** The frame's part, per change: from the splices' arrival to the frame painted, as the frame reports it. */
  rendering: Timings;
  /** From a change's first key press (or opening) to the frame painted. */
  painted: Timings;
}

/** The frame's end the editor uses: the render protocol, and the port it shares for components. */
export interface Frame {
  connection: FrameConnection;
  /** Sends the components to compile, on the same port as the splices, so they come first. */
  send(message: ComponentsMessage): void;
  /** Calls `listener` with each component the frame couldn't run. */
  onFailed(listener: (failed: ComponentFailed) => void): void;
}

/** A processor diagnostic as the renderer's rules for invalid text read it: the processed document's only. */
const forRenderer = (diagnostics: Diagnostic[]): NonNullable<Change["diagnostics"]> =>
  diagnostics.filter((d) => d.file === undefined).map((d) => ({ code: d.code, line: d.range.start.line, column: d.range.start.column }));

export class Editor {
  readonly state: EditorState = { diagnostics: [], problems: [], processing: new Timings(), rendering: new Timings(), painted: new Timings() };
  readonly #session: Session;
  readonly #frame: Frame;
  readonly #onChange: () => void;
  /** The text the last input left, and the selection before the next one. */
  #text = "";
  /** The text the diagnostics shown are of. */
  #processed: string | undefined;
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
    );
    frame.connection.on("rendered", ({ ms }) => {
      const since = this.#rendering.shift();
      if (since === undefined) return;
      this.state.painted.add(performance.now() - since);
      this.state.rendering.add(ms);
      onChange();
    });
    frame.connection.on("set", ({ input, value }) => {
      this.#inputs = { ...this.#inputs, [input]: value };
      this.#reopen();
    });
    frame.connection.on("error", ({ component, message }) => {
      if (component !== undefined) this.#problem(`${component}: ${message}`);
    });
    frame.onFailed(({ component, message }) => {
      this.#problem(`${component}: ${message}`);
    });
  }

  /** Opens `example`: its text is the text area's. Its nodes carry origins, which the renderer's keys come from. */
  open(example: Example, at = performance.now()): string {
    this.#request = { ...example.request, origins: true };
    this.#inputs = {};
    this.#text = example.request.files?.[example.request.path] ?? "";
    this.state.problems = [];
    this.#session.open(this.#request, this.#text, at);
    return this.#text;
  }

  /** Before an input: what is selected, which the input replaces. */
  beforeInput(selection: Selection): void {
    this.#selection = selection;
  }

  /** After an input, at `at`, the text area holds `text`. */
  input(text: string, at = performance.now()): void {
    const edit = editOf(this.#text, text, this.#selection);
    this.#text = text;
    this.#session.edit(edit, text, at);
  }

  /**
   * Makes a diagnostic's fix, as an input would, and gives the text after it for the text area. A fix is of the text
   * its diagnostic was found in, so after another input it does nothing until the diagnostics come again.
   */
  fix(fix: NonNullable<Diagnostic["fix"]>, at = performance.now()): string {
    if (this.#text !== this.#processed) return this.#text;
    const edit = fixOf(this.#text, fix);
    this.#text = this.#text.slice(0, edit.start) + edit.text + this.#text.slice(edit.end);
    this.#session.edit(edit, this.#text, at);
    return this.#text;
  }

  /** The document again, with the current text and the inputs' values. */
  #reopen(): void {
    const request = this.#request;
    if (!request) return;
    const files = { ...request.files, [request.path]: this.#text };
    this.#session.open({ ...request, files, inputs: { ...request.inputs, ...this.#inputs } }, this.#text);
  }

  #updated(update: Update): void {
    if (update.components) this.#frame.send({ type: "components", components: update.components });
    const change: Change = { text: update.text, diagnostics: forRenderer(update.diagnostics) };
    this.#rendering.push(update.since);
    this.#frame.connection.splice(update.splices, change, { replace: update.opened });
    this.state.diagnostics = update.diagnostics;
    this.#processed = update.text;
    this.state.processing.add(update.ms);
    if (update.components) this.state.problems = [];
    this.#onChange();
  }

  #problem(problem: string): void {
    if (!this.state.problems.includes(problem)) this.state.problems = [...this.state.problems, problem];
    this.#onChange();
  }
}
