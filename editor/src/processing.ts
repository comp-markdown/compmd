// The worker's part: it keeps the open document and answers each message with what it gives now.

import type { HostComponent, OpenDocument, Request } from "@compmd/processor";
import type { FromWorker, ToWorker } from "./protocol.ts";

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

export class Processing {
  readonly #open: (request: Request) => OpenDocument;
  #document: OpenDocument | undefined;

  /** Processes with documents `open` makes: `new OpenDocument(request)` in the worker. */
  constructor(open: (request: Request) => OpenDocument) {
    this.#open = open;
  }

  /** What `message` gives: the document's tree and diagnostics after it, and its components when they changed. */
  handle(message: ToWorker): FromWorker {
    const started = performance.now();
    try {
      const components = message.type === "open" ? this.#opened(message.request) : this.#edited(message.edits);
      const document = this.#document;
      if (!document) return { type: "failed", message: "no document is open" };
      return {
        type: "processed",
        expanded: document.expanded,
        diagnostics: [...document.diagnostics],
        ...(components ? { components } : {}),
        ms: performance.now() - started,
      };
    } catch (error) {
      return { type: "failed", message: messageOf(error) };
    }
  }

  #opened(request: Request): readonly HostComponent[] {
    this.#document?.free();
    this.#document = undefined;
    this.#document = this.#open(request);
    return this.#document.components;
  }

  /** The components after `edits`, when one of them changed them. */
  #edited(edits: { start: number; end: number; text: string }[]): readonly HostComponent[] | undefined {
    let changed = false;
    for (const { start, end, text } of edits) changed = Boolean(this.#document?.edit(start, end, text).components) || changed;
    return changed ? this.#document?.components : undefined;
  }
}
