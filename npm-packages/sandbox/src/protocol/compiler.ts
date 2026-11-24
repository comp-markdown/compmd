// The parent's end for the compile frame, a strict frame that runs no components: sources go in, compiled text comes
// out, which the parent gives the frames that run it.

import { afterAwake } from "./awake.ts";
import { FROM_FRAME, isMessage, SANDBOX_VERSION, type CompileRequest, type FromFrame, type Port, type ToFrame } from "./messages.ts";
import { failureText, LOAD_MS } from "./parent-end.ts";

export interface CompilerOptions {
  /**
   * How long the frame may take to say `ready`, in ms (at least `LOAD_MS`), and the URL of its document: a frame that
   * hasn't by then never loaded its runtime, and every request fails with why, so no component waits for its code
   * forever.
   */
  ready?: { timeout: number; url: string };
}

export interface Compiler {
  /**
   * `request` compiled.
   *
   * @throws Error when the source doesn't compile, with the compiler's message.
   */
  compile(request: CompileRequest): Promise<{ code: string; css?: string }>;
  /** Fails every request with `error`, those waiting and those after: the frame can't answer them. */
  fail(error: Error): void;
  close(): void;
}

/**
 * How long a compile may take once sent, in ms of this thread's awake time: then it fails, and its component's uses
 * show their fallbacks, instead of waiting for a compile frame that stopped answering (docs/sandbox-protocol.md,
 * decision 6). The compile frame isn't pinged: this stands for it.
 */
export const COMPILE_MS = 30_000;

interface Pending {
  resolve(compiled: { code: string; css?: string }): void;
  reject(error: Error): void;
  /** Stops its timeout, once it is answered. */
  cancel(): void;
}

/**
 * Connects to the compile frame on `port`. Requests wait for its `ready`, and each is matched to its answer by an id of
 * its own, so two components of one name, a document's and an example's (spec §7.8), each get their own code. Each
 * fails after `COMPILE_MS` without an answer. With `options.ready`, a frame that never says `ready` fails every
 * request, those waiting and those after.
 *
 * @example
 * const compiler = connectCompiler(port, { ready: { timeout: 5000, url: strictUrl } });
 * const { code } = await compiler.compile({ name: "chart", language: "jsx", code: source });
 */
export function connectCompiler(port: Port, options: CompilerOptions = {}): Compiler {
  const pending = new Map<number, Pending>();
  let lastId = 0;
  let waiting: { id: number; request: CompileRequest }[] | undefined = [];
  /** Why every request fails, once the frame didn't load. */
  let unloaded: Error | undefined;
  const send = (id: number, request: CompileRequest) => {
    const asked = pending.get(id);
    if (!asked) return;
    asked.cancel = afterAwake(COMPILE_MS, () => {
      pending.delete(id);
      asked.reject(new Error(`the compile frame didn't answer for \`${request.name}\` within ${String(COMPILE_MS / 1000)} s`));
    });
    port.postMessage({ type: "compile", id, ...request } satisfies ToFrame);
  };
  const fail = (error: Error) => {
    unloaded = error;
    waiting = undefined;
    for (const answer of pending.values()) {
      answer.cancel();
      answer.reject(error);
    }
    pending.clear();
  };
  const { ready } = options;
  // This thread's awake time: a busy thread the frame shares keeps it from loading (`awake.ts`).
  const cancelUnloaded = ready
    ? afterAwake(Math.max(ready.timeout, LOAD_MS), () => {
        if (waiting) fail(new Error(`the compile frame ${failureText({ reason: "unloaded" }, ready.url)}`));
      })
    : () => undefined;
  const listener = ({ data }: { data: unknown }) => {
    if (!isMessage<FromFrame>(data, FROM_FRAME)) return;
    if (data.type === "ready" && waiting) {
      cancelUnloaded();
      // A frame document of another deploy: said so, not left to look as if it never loaded.
      if (data.sandbox !== SANDBOX_VERSION) {
        fail(new Error(`the compile frame ${failureText({ reason: "version", version: data.sandbox }, ready?.url ?? "")}`));
        return;
      }
      port.postMessage({ type: "level", level: "strict" } satisfies ToFrame);
      const queued = waiting;
      waiting = undefined;
      for (const { id, request } of queued) send(id, request);
    }
    if (data.type !== "compiled" && data.type !== "failed") return;
    // Only the request of its own id: an answer to none, or to one that timed out, is dropped.
    const answer = pending.get(data.id);
    if (!answer) return;
    pending.delete(data.id);
    answer.cancel();
    if (data.type === "failed") answer.reject(new Error(data.message));
    else answer.resolve({ code: data.code, ...(data.css === undefined ? {} : { css: data.css }) });
  };
  port.addEventListener("message", listener);
  port.start?.();
  return {
    compile: (request) =>
      new Promise((resolve, reject) => {
        if (unloaded) {
          reject(unloaded);
          return;
        }
        const id = ++lastId;
        pending.set(id, { resolve, reject, cancel: () => undefined });
        if (waiting) waiting.push({ id, request });
        else send(id, request);
      }),
    fail,
    close: () => {
      cancelUnloaded();
      for (const answer of pending.values()) answer.cancel();
      port.removeEventListener("message", listener);
    },
  };
}
