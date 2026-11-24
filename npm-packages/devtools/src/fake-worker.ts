// A worker the tests answer by hand, as the session's, the editor's and the views' clients use one.

import type { WorkerPort } from "./worker-link.ts";

/** A worker that records what it is sent; the test answers, fails or ends it. */
export class FakeWorker<Sent, Received> implements WorkerPort<Sent, Received> {
  readonly received: Sent[] = [];
  terminated = false;
  #onMessage: ((event: { data: Received }) => void) | undefined;
  #onError: ((event: { message?: string }) => void) | undefined;

  postMessage(message: Sent): void {
    this.received.push(message);
  }

  addEventListener(type: "message", listener: (event: { data: Received }) => void): void;
  addEventListener(type: "error", listener: (event: { message?: string }) => void): void;
  addEventListener(type: "message" | "error", listener: ((event: { data: Received }) => void) | ((event: { message?: string }) => void)): void {
    if (type === "message") this.#onMessage = listener as (event: { data: Received }) => void;
    else this.#onError = listener as (event: { message?: string }) => void;
  }

  terminate(): void {
    this.terminated = true;
  }

  /** The worker answers with `data`. */
  answer(data: Received): void {
    this.#onMessage?.({ data });
  }

  /** The worker's script throws `message`. */
  fail(message: string): void {
    this.#onError?.({ message });
  }
}
