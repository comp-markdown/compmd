// The editor's loop: each input in the text area becomes an edit, the worker processes it, and the page's sandbox
// applies the splices it gave, which the editor passes on unread. An input component's `set` on the page, or the inputs
// panel, opens the document again with the new value, and so does remote data the reader allowed and the page fetched.
// It times each change from its first key press to the page painted.

import type { Diagnostic, ExpandedInput, PositionalHint, Request } from "@compmd/processor";
import type { Change } from "@compmd/render";
import { loadData, type View } from "@compmd/sandbox";
import type { DevtoolsDocument } from "./document.ts";
import { editOf, fixOf, type Selection } from "./edit.ts";
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
  /** The built modules fetched from the registry, by `pack@version/path`, which the page's sandbox loads as code. */
  readonly #modules: Map<string, Blob>;
  /** What couldn't be fetched from the registry, listed with the components' problems. */
  #unfetched: string[] = [];
  /** The problem each use that failed is listed for, by its `use`, until it draws again. */
  readonly #failures = new Map<string, string>();
  /** The text the last input left, and the selection before the next one. */
  #text = "";
  /** The text the diagnostics shown are of, and the one the hint is of. */
  #processed: string | undefined;
  #hinted: string | undefined;
  #selection: Selection = { start: 0, end: 0 };
  #request: Request | undefined;
  /** The origins the document came with, which the reader is never asked about. */
  #configured: readonly string[] = [];
  readonly #load: typeof loadData;
  #inputs: Record<string, unknown> = {};
  /** When each render sent to the frame started, for its paint. */
  readonly #rendering: number[] = [];

  /**
   * Runs with `worker` and `frame`; `onChange` is called when the state changes. The built modules fetched from the
   * registry go in `modules`, where the page's sandbox finds them. `load` asks the reader about the remote data's new
   * origins and fetches it (`loadData` by default).
   */
  constructor(worker: WorkerPort, frame: Frame, onChange: () => void, modules = new Map<string, Blob>(), load: typeof loadData = loadData) {
    this.#frame = frame;
    this.#onChange = onChange;
    this.#modules = modules;
    this.#load = load;
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
      // Packs the document imports came from the registry: it opens again with them.
      ({ arrived, problems }) => {
        this.#unfetched = [...this.#unfetched, ...problems.filter((problem) => !this.#unfetched.includes(problem))];
        for (const problem of problems) this.#problem(problem);
        if (arrived) this.#reopen();
      },
      (urls) => {
        void this.#loadRemote(urls);
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
    // A component that couldn't compile, load or run (W121, W118), or isn't allowed where it is used; a page's own
    // problem has none. A use that failed and draws again takes its problem off the list.
    frame.on("error", ({ component, code, message, use }) => {
      if (!component) return;
      const problem = `${code ? `${code} ` : ""}${component}: ${message}`;
      if (use !== undefined) this.#failures.set(use, problem);
      this.#problem(problem);
    });
    frame.on("recovered", ({ use }) => {
      const problem = this.#failures.get(use);
      this.#failures.delete(use);
      if (problem === undefined || [...this.#failures.values()].includes(problem)) return;
      this.state.problems = this.state.problems.filter((listed) => listed !== problem);
      onChange();
    });
  }

  /** Opens `document`: its text is the text area's. Its nodes carry origins, which the renderer's keys come from. */
  open({ request }: DevtoolsDocument, at = performance.now()): string {
    this.#request = { ...request, origins: true };
    this.#configured = request.allowedOrigins ?? [];
    this.#inputs = {};
    this.#text = request.files?.[request.path] ?? "";
    this.#unfetched = [];
    this.#failures.clear();
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

  /**
   * The remote data the document needs at `urls`: the reader is asked about their new origins, in the page, and what
   * the allowed ones gave opens the document again with it (origins are decided when it is validated). Data the
   * document came with isn't fetched again.
   */
  async #loadRemote(urls: readonly string[]): Promise<void> {
    const request = this.#request;
    if (!request) return;
    const needed = urls.filter((url) => !(url in (request.remote ?? {})));
    if (needed.length === 0) return;
    const { allowedOrigins, remote } = await this.#load(needed, { configured: this.#configured, document: request.path });
    const current = this.#request;
    if (current !== request) return;
    const known = new Set(request.allowedOrigins ?? []);
    const newOrigins = allowedOrigins.filter((origin) => !known.has(origin));
    if (newOrigins.length === 0 && Object.keys(remote).length === 0) return;
    this.#request = { ...request, allowedOrigins: [...known, ...newOrigins], remote: { ...request.remote, ...remote } };
    this.#reopen();
  }

  /** The document again, with the current text and the inputs' values. */
  #reopen(): void {
    const document = this.document;
    if (document) this.#session.open(document, this.#text);
  }

  #updated(update: Update): void {
    // Before the components reach the frame, so the sandbox finds their modules.
    for (const [place, code] of Object.entries(update.modules ?? {})) this.#modules.set(place, code);
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
    if (update.components) {
      this.state.problems = [...this.#unfetched];
      this.#failures.clear();
    }
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
