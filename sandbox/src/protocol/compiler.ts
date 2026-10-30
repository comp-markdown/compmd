// The parent's end for the compile frame, a strict frame that runs no components: sources go in, compiled text comes
// out, which the parent gives the frames that run it.

import { FROM_FRAME, isMessage, SANDBOX_VERSION, type CompileRequest, type FromFrame, type Port, type ToFrame } from "./messages.ts";

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
 * of an older one still waiting, which then never settles.
 *
 * @example
 * const compiler = connectCompiler(port);
 * const { code } = await compiler.compile({ name: "chart", language: "jsx", code: source });
 */
export function connectCompiler(port: Port): Compiler {
  const pending = new Map<string, Pending>();
  let waiting: CompileRequest[] | undefined = [];
  const send = (request: CompileRequest) => {
    port.postMessage({ type: "compile", ...request } satisfies ToFrame);
  };
  const listener = ({ data }: { data: unknown }) => {
    if (!isMessage<FromFrame>(data, FROM_FRAME)) return;
    if (data.type === "ready" && waiting && data.sandbox === SANDBOX_VERSION) {
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
        pending.set(request.name, { resolve, reject });
        if (waiting) waiting.push(request);
        else send(request);
      }),
    close: () => {
      port.removeEventListener("message", listener);
    },
  };
}
