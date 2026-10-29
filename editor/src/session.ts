// The editor's side of processing: it sends each edit to the worker, and edits made while the worker is busy go
// together in the next message. Each answer is an update for the frame, with the text it is of and, when it came
// from one edit, that edit's range, which the renderer's keys follow.

import type { Diagnostic, HostComponent, Request } from "@compmd/processor";
import type { EditRange, ExpandedRoot } from "@compmd/render";
import type { Edit } from "./edit.ts";
import type { FromWorker, ToWorker } from "./protocol.ts";

/** What the session needs of a worker: a `Worker` is one. */
export interface WorkerPort {
  postMessage(message: ToWorker): void;
  addEventListener(type: "message", listener: (event: { data: FromWorker }) => void): void;
}

/** A processed state of the document, for the frame. */
export interface Update {
  expanded: ExpandedRoot;
  diagnostics: Diagnostic[];
  /** The host-language components, when they changed. */
  components?: readonly HostComponent[];
  /** The text the tree is of. */
  text: string;
  /** The range of the text before it that one edit replaced; absent after opening and for merged edits. */
  edit?: EditRange;
  /** When the first change it covers was made (`performance.now()`): opening, or the first key press. */
  since: number;
  /** How long the worker took. */
  ms: number;
}

/** A message sent and not answered yet. */
interface Sent {
  /** The document it is for: a later `open` makes earlier answers stale. */
  generation: number;
  text: string;
  edit?: EditRange;
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
  readonly #sent: Sent[] = [];
  #waiting: Waiting[] = [];
  #generation = 0;

  constructor(worker: WorkerPort, onUpdate: (update: Update) => void, onFailure: (message: string) => void) {
    this.#worker = worker;
    this.#onUpdate = onUpdate;
    this.#onFailure = onFailure;
    worker.addEventListener("message", ({ data }) => {
      this.#answered(data);
    });
  }

  /** Opens `request`, whose document's text is `text`, in place of the one before; edits not sent yet are dropped. */
  open(request: Request, text: string, at = performance.now()): void {
    this.#generation++;
    this.#waiting = [];
    this.#send({ type: "open", request }, { generation: this.#generation, text, since: at });
  }

  /** `edit` was made at `at`, giving `text`. */
  edit(edit: Edit, text: string, at = performance.now()): void {
    this.#waiting.push({ edit, text, at });
    if (this.#sent.length === 0) this.#flush();
  }

  #flush(): void {
    const batch = this.#waiting;
    const [first] = batch;
    const last = batch.at(-1);
    if (!first || !last) return;
    this.#waiting = [];
    const single = batch.length === 1 ? { edit: { start: first.edit.start, end: first.edit.end } } : {};
    this.#send({ type: "edits", edits: batch.map((waiting) => waiting.edit) }, { generation: this.#generation, text: last.text, since: first.at, ...single });
  }

  #send(message: ToWorker, sent: Sent): void {
    this.#sent.push(sent);
    this.#worker.postMessage(message);
  }

  #answered(answer: FromWorker): void {
    const sent = this.#sent.shift();
    if (sent?.generation === this.#generation) {
      if (answer.type === "failed") this.#onFailure(answer.message);
      else {
        const { expanded, diagnostics, components, ms } = answer;
        this.#onUpdate({ expanded, diagnostics, text: sent.text, since: sent.since, ms, ...(components ? { components } : {}), ...(sent.edit ? { edit: sent.edit } : {}) });
      }
    }
    if (this.#sent.length === 0) this.#flush();
  }
}
