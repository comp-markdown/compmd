// The document's trees (spec §12), for the editor's tabs: the parse tree and the expanded tree, as JSON. The editor's
// worker keeps the document open and gives only what changed, so the trees come from a second worker that processes
// the document from scratch, and only while a tab shows them.

import { process, type Request } from "@compmd/processor";

/** From the editor to the trees' worker. */
export interface TreesRequest {
  request: Request;
}

/** The trees of a document, each as indented JSON, or why it couldn't be processed. */
export type Trees = { parse: string; expanded: string } | { failed: string };

/** What the trees' worker does with each request: processes the document from scratch. */
export function treesOf({ request }: TreesRequest): Trees {
  try {
    const { parse, expanded } = process(request);
    return { parse: JSON.stringify(parse, null, 2), expanded: JSON.stringify(expanded, null, 2) };
  } catch (error) {
    return { failed: error instanceof Error ? error.message : String(error) };
  }
}

/** What the editor needs of the trees' worker: a `Worker` is one. */
export interface TreesPort {
  postMessage(message: TreesRequest): void;
  addEventListener(type: "message", listener: (event: { data: Trees }) => void): void;
}

/**
 * Asks the trees' worker for a document's trees, one request at a time: a request made while it is busy waits, and a
 * later one takes its place, since only the latest text matters.
 */
export class TreesClient {
  readonly #worker: TreesPort;
  #busy = false;
  #waiting: Request | undefined;

  /** Asks `worker`, and calls `onTrees` with each answer. */
  constructor(worker: TreesPort, onTrees: (trees: Trees) => void) {
    this.#worker = worker;
    worker.addEventListener("message", ({ data }) => {
      this.#busy = false;
      onTrees(data);
      this.#send();
    });
  }

  /** Asks for the trees of `request`'s document. */
  request(request: Request): void {
    this.#waiting = request;
    if (!this.#busy) this.#send();
  }

  #send(): void {
    const request = this.#waiting;
    if (!request) return;
    this.#waiting = undefined;
    this.#busy = true;
    this.#worker.postMessage({ request });
  }
}
