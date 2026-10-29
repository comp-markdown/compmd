// The worker's part: it keeps the open document and answers each message with what it did, each splice as the
// processor's JSON text, which goes to the frame unread.

import type { ChangesJson, OpenDocumentJson, Request } from "@compmd/processor";
import type { FromWorker, ToWorker } from "./protocol.ts";

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export class Processing {
  readonly #open: (request: Request) => OpenDocumentJson;
  #document: OpenDocumentJson | undefined;

  /** Processes with documents `open` makes: `new OpenDocumentJson(request)` in the worker. */
  constructor(open: (request: Request) => OpenDocumentJson) {
    this.#open = open;
  }

  /** What `message` gives: a splice per edit, the diagnostics after them, and the components when they changed. */
  handle(message: ToWorker): FromWorker {
    const started = performance.now();
    try {
      const changes = message.type === "open" ? [this.#opened(message.request)] : this.#edited(message.edits);
      const document = this.#document;
      if (!document) return { type: "failed", message: "no document is open" };
      const components = changes.findLast((change) => change.components)?.components;
      return {
        type: "processed",
        splices: changes.map((change) => change.splice),
        opened: message.type === "open",
        diagnostics: (changes.at(-1) ?? document.changes()).diagnostics,
        ...(components ? { components } : {}),
        ms: performance.now() - started,
      };
    } catch (error) {
      return { type: "failed", message: messageOf(error) };
    }
  }

  #opened(request: Request): ChangesJson {
    this.#document?.free();
    this.#document = undefined;
    this.#document = this.#open(request);
    return this.#document.changes();
  }

  #edited(edits: readonly { start: number; end: number; text: string }[]): ChangesJson[] {
    const document = this.#document;
    if (!document) return [];
    return edits.map(({ start, end, text }) => document.edit(start, end, text));
  }
}
