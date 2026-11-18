// The parent's end for the compile frame, a strict frame that runs no components: sources go in, compiled text comes
// out, which the parent gives the frames that run it.

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
  close(): void;
}

interface Pending {
  resolve(compiled: { code: string; css?: string }): void;
  reject(error: Error): void;
}

/**
 * Connects to the compile frame on `port`. Requests wait for its `ready`; a newer request for a name answers in place
 * of an older one still waiting, which then never settles. With `options.ready`, a frame that never says it fails
 * every request, those waiting and those after.
 *
 * @example
 * const compiler = connectCompiler(port, { ready: { timeout: 5000, url: strictUrl } });
 * const { code } = await compiler.compile({ name: "chart", language: "jsx", code: source });
 */
export function connectCompiler(port: Port, options: CompilerOptions = {}): Compiler {
  const pending = new Map<string, Pending>();
  let waiting: CompileRequest[] | undefined = [];
  /** Why every request fails, once the frame didn't load. */
  let unloaded: Error | undefined;
  const send = (request: CompileRequest) => {
    port.postMessage({ type: "compile", ...request } satisfies ToFrame);
  };
  const { ready } = options;
  const timer = ready
    ? setTimeout(() => {
        if (!waiting) return;
        unloaded = new Error(`the compile frame ${failureText({ reason: "unloaded" }, ready.url)}`);
        waiting = undefined;
        for (const answer of pending.values()) answer.reject(unloaded);
        pending.clear();
      }, Math.max(ready.timeout, LOAD_MS))
    : undefined;
  const listener = ({ data }: { data: unknown }) => {
    if (!isMessage<FromFrame>(data, FROM_FRAME)) return;
    if (data.type === "ready" && waiting && data.sandbox === SANDBOX_VERSION) {
      clearTimeout(timer);
      port.postMessage({ type: "level", level: "strict" } satisfies ToFrame);
      for (const request of waiting) send(request);
      waiting = undefined;
    }
    if (data.type !== "compiled" && data.type !== "failed") return;
    const answer = pending.get(data.name);
    pending.delete(data.name);
    if (data.type === "failed") answer?.reject(new Error(data.message));
    else answer?.resolve({ code: data.code, ...(data.css === undefined ? {} : { css: data.css }) });
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
        pending.set(request.name, { resolve, reject });
        if (waiting) waiting.push(request);
        else send(request);
      }),
    close: () => {
      clearTimeout(timer);
      port.removeEventListener("message", listener);
    },
  };
}
