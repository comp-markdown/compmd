// The parent's end of the protocol, one per frame it made: it waits for the frame's `ready`, gives it its level first,
// checks what the frame sends against the part it gave it, and stops a frame that stops answering.

import { FROM_FRAME, isMessage, SANDBOX_VERSION, type ComponentCode, type ErrorReason, type FromFrame, type Level, type Part, type Port, type ToFrame } from "./messages.ts";

export interface ChildOptions {
  level: Level;
  /** A permissive frame's second policy, with its pack's origins. */
  policy?: string;
  /** Whether a component in the frame may set `input`: one of the frame's uses binds a param to it. */
  binds(input: string): boolean;
  /** The port for `host`, for a permissive frame of a pack given `host`. */
  host?: MessagePort;
  /** How often to ping the frame, and how long it may not answer before it is stopped, in ms. */
  ping?: { every: number; timeout: number };
}

/** Why the parent stopped listening to a frame. */
export type ChildFailure = { reason: "version"; version: number } | { reason: "hung" };

export interface ChildEvents {
  set: { input: string; value: unknown };
  error: { reason: ErrorReason; component?: string; message: string };
  link: { url: string };
  loaded: Record<string, never>;
  rendered: { ms: number };
  size: { width: number; height: number };
  /** The frame is of another version, or stopped answering: the parent removes it, and its use shows its fallback. */
  failed: ChildFailure;
}

export interface ChildEnd {
  code(components: ComponentCode[]): void;
  part(part: Part): void;
  styles(urls: string[]): void;
  /** Sends a message of `@compmd/render`'s protocol, for a page frame, once the frame has its level. */
  send(message: unknown): void;
  on<T extends keyof ChildEvents>(type: T, listener: (event: ChildEvents[T]) => void): void;
  close(): void;
}

type Listeners = { [T in keyof ChildEvents]: ((event: ChildEvents[T]) => void)[] };

/**
 * Connects to the frame on `port`, the end the parent kept of the frame's port.
 *
 * @throws Error when `host` is given to a strict frame.
 * @example
 * const child = connectChild(port, { level: "strict", binds: (input) => bound.has(input) });
 * child.code(components);
 * child.part({ node, inputs });
 * child.on("set", ({ input, value }) => setInput(input, value));
 */
export function connectChild(port: Port, options: ChildOptions): ChildEnd {
  if (options.host && options.level !== "permissive") throw new Error("only a permissive frame is given host");
  const listeners: Listeners = { set: [], error: [], link: [], loaded: [], rendered: [], size: [], failed: [] };
  const emit = <T extends keyof ChildEvents>(type: T, event: ChildEvents[T]) => {
    for (const listener of listeners[type]) listener(event);
  };
  /** What waits for the frame's `ready`; `undefined` once it came. */
  let waiting: (() => void)[] | undefined = [];
  let closed = false;
  const whenReady = (send: () => void) => {
    if (closed) return;
    if (waiting) waiting.push(send);
    else send();
  };
  /** Sends a sandbox message, or one of `@compmd/render`'s, once the frame is ready. */
  const post = (message: unknown) => {
    whenReady(() => {
      port.postMessage(message);
    });
  };

  let pings: ReturnType<typeof setInterval> | undefined;
  let sent = 0;
  let answered = 0;
  let lastAnswer = 0;
  const stop = (failure: ChildFailure) => {
    close();
    emit("failed", failure);
  };
  const startPings = ({ every, timeout }: { every: number; timeout: number }) => {
    lastAnswer = Date.now();
    pings = setInterval(() => {
      if (answered < sent && Date.now() - lastAnswer > timeout) {
        stop({ reason: "hung" });
        return;
      }
      sent += 1;
      port.postMessage({ type: "ping", n: sent } satisfies ToFrame);
    }, every);
  };

  const listener = ({ data }: { data: unknown }) => {
    if (!isMessage<FromFrame>(data, FROM_FRAME)) return;
    if (data.type === "ready") {
      if (!waiting) return;
      if (data.sandbox !== SANDBOX_VERSION) {
        stop({ reason: "version", version: data.sandbox });
        return;
      }
      port.postMessage({ type: "level", level: options.level, ...(options.policy === undefined ? {} : { policy: options.policy }) } satisfies ToFrame);
      if (options.host) port.postMessage({ type: "host" } satisfies ToFrame, [options.host]);
      const queued = waiting;
      waiting = undefined;
      for (const send of queued) send();
      if (options.ping) startPings(options.ping);
      return;
    }
    if (waiting) return;
    switch (data.type) {
      case "pong":
        answered = Math.max(answered, data.n);
        lastAnswer = Date.now();
        break;
      case "set":
        // A frame may set only what one of its uses is bound to; strict code may have written this message.
        if (options.binds(data.input)) emit("set", { input: data.input, value: data.value });
        break;
      case "error":
        emit("error", { reason: data.reason, message: data.message, ...(data.component === undefined ? {} : { component: data.component }) });
        break;
      case "link":
        emit("link", { url: data.url });
        break;
      case "loaded":
        emit("loaded", {});
        break;
      case "rendered":
        emit("rendered", { ms: data.ms });
        break;
      case "size":
        emit("size", { width: data.width, height: data.height });
        break;
      case "compiled":
      case "failed":
        // The compile frame's; its own parent end reads them (`connectCompiler`).
        break;
    }
  };
  port.addEventListener("message", listener);
  port.start?.();

  function close() {
    closed = true;
    if (pings !== undefined) clearInterval(pings);
    port.removeEventListener("message", listener);
  }

  return {
    code: (components) => {
      post({ type: "code", components } satisfies ToFrame);
    },
    part: (part) => {
      post({ type: "part", part } satisfies ToFrame);
    },
    styles: (urls) => {
      post({ type: "styles", urls } satisfies ToFrame);
    },
    send: (message) => {
      post(message);
    },
    on: (type, listener) => {
      listeners[type].push(listener);
    },
    close,
  };
}
