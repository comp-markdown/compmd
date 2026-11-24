// The editor's link to one of its workers (audit B9, A2): the editor's worker, which keeps the document open, and the
// views' worker. Both clients send one message at a time and wait for its answer; the link hears that the worker died,
// so a client never waits for good ("Processing…"), and ends the worker when the Developer Tools close.

/** What a client needs of a worker: a `Worker` is one. */
export interface WorkerPort<Sent, Received> {
  postMessage(message: Sent): void;
  addEventListener(type: "message", listener: (event: { data: Received }) => void): void;
  addEventListener(type: "error", listener: (event: { message?: string }) => void): void;
  terminate(): void;
}

/** A worker as a client uses it: messages sent, answers and its failure heard, and ended once. */
export class WorkerLink<Sent, Received> {
  readonly #port: WorkerPort<Sent, Received>;
  #closed = false;

  /**
   * Links `port`: `onMessage` hears each answer, and `onFailure` that the worker failed (its script threw, or it
   * couldn't start), with why.
   */
  constructor(port: WorkerPort<Sent, Received>, onMessage: (data: Received) => void, onFailure: (message: string) => void) {
    this.#port = port;
    port.addEventListener("message", ({ data }) => {
      if (!this.#closed) onMessage(data);
    });
    port.addEventListener("error", (event) => {
      if (!this.#closed) onFailure(`the worker stopped: ${event.message ?? "it failed to start or threw"}`);
    });
  }

  /** Sends `message`, unless the link is closed. */
  post(message: Sent): void {
    if (!this.#closed) this.#port.postMessage(message);
  }

  /** Ends the worker; nothing more is sent or heard. */
  close(): void {
    if (this.#closed) return;
    this.#closed = true;
    this.#port.terminate();
  }
}
