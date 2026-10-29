// The editor's side of processing: it sends each edit to the worker, and edits made while the worker is busy go
// together in the next message. Each answer is an update for the frame: a splice per edit, each with its edit, which
// the renderer's keys follow, and the text they leave. Each message carries the caret, so each answer brings the hint
// at it; a caret that moves while nothing is typed asks for the hint alone.

import type { Diagnostic, ExpandedInput, HostComponent, PositionalHint, Request } from "@compmd/processor";
import type { Edit } from "./edit.ts";
import type { FromWorker, ToWorker } from "./protocol.ts";

/** What the session needs of a worker: a `Worker` is one. */
export interface WorkerPort {
  postMessage(message: ToWorker): void;
  addEventListener(type: "message", listener: (event: { data: FromWorker }) => void): void;
}

/** A processed state of the document, for the frame. */
export interface Update {
  /** What each edit did to the expanded tree, as the processor's JSON text. */
  splices: string[];
  /** Whether the document was opened: its splice replaces the page. */
  opened: boolean;
  diagnostics: Diagnostic[];
  /** The document's inputs, by name, with their values. */
  inputs: Record<string, ExpandedInput>;
  /** The host-language components, when they changed. */
  components?: readonly HostComponent[];
  /** What is at the caret in `text`. */
  hint?: PositionalHint;
  /** The text after the splices. */
  text: string;
  /** When the first change it covers was made (`performance.now()`): opening, or the first key press. */
  since: number;
  /** How long the worker took. */
  ms: number;
}

/** The hint at the caret, and the text it is of. */
export interface Hinted {
  hint?: PositionalHint;
  text: string;
}

/** A message sent and not answered yet. */
interface Sent {
  /** The document it is for: a later `open` makes earlier answers stale. */
  generation: number;
  text: string;
  since: number;
}

/** An edit waiting for the worker, with the text after it. */
interface Waiting {
  edit: Edit;
  text: string;
  at: number;
}

export class Session {
  readonly #worker: WorkerPort;
  readonly #onUpdate: (update: Update) => void;
  readonly #onFailure: (message: string) => void;
  readonly #onHint: (hinted: Hinted) => void;
  readonly #sent: Sent[] = [];
  #waiting: Waiting[] = [];
  #generation = 0;
  /** The text the last open or edit left. */
  #text = "";
  /** Where the caret is, and where it was when the last message was sent. */
  #caret = 0;
  #asked = 0;

  constructor(worker: WorkerPort, onUpdate: (update: Update) => void, onFailure: (message: string) => void, onHint: (hinted: Hinted) => void) {
    this.#worker = worker;
    this.#onUpdate = onUpdate;
    this.#onFailure = onFailure;
    this.#onHint = onHint;
    worker.addEventListener("message", ({ data }) => {
      this.#answered(data);
    });
  }

  /** Opens `request`, whose document's text is `text`, in place of the one before; edits not sent yet are dropped. */
  open(request: Request, text: string, at = performance.now()): void {
    this.#generation++;
    this.#waiting = [];
    this.#text = text;
    this.#send({ type: "open", request, caret: this.#caret }, { generation: this.#generation, text, since: at });
  }

  /** `edit` was made at `at`, giving `text`, with the caret at `caret`. */
  edit(edit: Edit, text: string, caret: number, at = performance.now()): void {
    this.#waiting.push({ edit, text, at });
    this.#text = text;
    this.#caret = caret;
    if (this.#sent.length === 0) this.#flush();
  }

  /** The caret moved to `caret` (UTF-16 units) without an edit. */
  moveCaret(caret: number): void {
    this.#caret = caret;
    if (this.#sent.length === 0) this.#flush();
  }

  #flush(): void {
    const batch = this.#waiting;
    const [first] = batch;
    const last = batch.at(-1);
    if (first && last) {
      this.#waiting = [];
      this.#send({ type: "edits", edits: batch.map((waiting) => waiting.edit), caret: this.#caret }, { generation: this.#generation, text: last.text, since: first.at });
    } else if (this.#caret !== this.#asked) {
      this.#send({ type: "hint", caret: this.#caret }, { generation: this.#generation, text: this.#text, since: performance.now() });
    }
  }

  #send(message: ToWorker, sent: Sent): void {
    this.#sent.push(sent);
    this.#asked = message.caret;
    this.#worker.postMessage(message);
  }

  #answered(answer: FromWorker): void {
    const sent = this.#sent.shift();
    if (sent?.generation === this.#generation) {
      if (answer.type === "failed") this.#onFailure(answer.message);
      else if (answer.type === "hint") this.#onHint({ text: sent.text, ...(answer.hint ? { hint: answer.hint } : {}) });
      else {
        const { splices, opened, diagnostics, inputs, components, hint, ms } = answer;
        this.#onUpdate({ splices, opened, diagnostics, inputs, text: sent.text, since: sent.since, ms, ...(components ? { components } : {}), ...(hint ? { hint } : {}) });
      }
    }
    if (this.#sent.length === 0) this.#flush();
  }
}
