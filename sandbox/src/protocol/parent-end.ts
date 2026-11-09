// The parent's end of the protocol, one per frame it made: it waits for the frame's `ready`, gives it its level first,
// checks what the frame sends against the part it gave it, and stops a frame that stops answering.

import { BUSY_MS } from "@compmd/render";
import { FROM_FRAME, isChange, isMessage, SANDBOX_VERSION, type ComponentCode, type ErrorReason, type FromFrame, type Level, type Part, type Port, type Scheme, type ToFrame } from "./messages.ts";

export interface ChildOptions {
  level: Level;
  /** A permissive frame's second policy, with its pack's origins. */
  policy?: string;
  /** Whether a component in the frame may set `input`: one of the frame's uses binds a param to it. */
  binds(input: string): boolean;
  /** The port for `host`, for a permissive frame of a pack given `host`. */
  host?: MessagePort;
  /**
   * How often to ping the frame, and how long it may not answer before it is stopped, in ms. A frame that hasn't said
   * `ready` that long after it connected, and at least `LOAD_MS`, never loaded its runtime, and is stopped too.
   */
  ping?: { every: number; timeout: number };
  /**
   * The file of an image a page frame asks for by its `src`, or `undefined` for none. Strict code may have asked, so
   * it gives only images of the page it sent. Without it, the frame gets no images.
   */
  image?(url: string): Promise<Blob | undefined>;
}

/**
 * How long a frame may take to load its runtime and say `ready` at least, in ms, whatever its `timeout` for answering:
 * loading takes longer than answering a ping, on a slow device most of all.
 */
export const LOAD_MS = 5000;

/**
 * Why the parent stopped listening to a frame: it is of another protocol version, it stopped answering, or it never
 * said `ready`, since its runtime didn't load (refused by its policy, not found, or the network failed).
 */
export type ChildFailure = { reason: "version"; version: number } | { reason: "hung" } | { reason: "unloaded" };

/**
 * What happened to a frame of the document at `url`, after "its frame" or "the page's frame", for an `error`'s
 * message.
 *
 * @example
 * `its frame ${failureText({ reason: "unloaded" }, url)}`; // "its frame didn't load its runtime from app://localhost/frames/strict.html"
 */
export function failureText(failure: ChildFailure, url: string): string {
  switch (failure.reason) {
    case "hung":
      return "stopped answering";
    case "version":
      return `is of protocol version ${String(failure.version)}`;
    case "unloaded":
      return `didn't load its runtime from ${url}: is the frame document served there, and does its policy allow the files next to it?`;
  }
}

export interface ChildEvents {
  set: { input: string; value: unknown };
  error: { reason: ErrorReason; component?: string; use?: string; message: string };
  /** A use that failed draws again, as the frame says: code of its level may have said it, so it only clears a listing. */
  recovered: { component: string; use: string };
  link: { url: string };
  loaded: Record<string, never>;
  rendered: { ms: number };
  size: { width: number; height: number };
  /**
   * The frame is of another version, stopped answering or never loaded its runtime: the parent removes it, and its
   * use shows its fallback.
   */
  failed: ChildFailure;
  /** What the frame's observer reports, unchecked (`observedOf` checks each event); only while the parent asks. */
  observed: { events: unknown[] };
  /** The frame set an input none of its uses binds, and the parent dropped it. */
  refused: { input: string };
}

/** The most events a parent takes from one `observed` message; a frame that has more reports them later. */
const BATCH = 2000;

export interface ChildEnd {
  code(components: ComponentCode[]): void;
  part(part: Part): void;
  styles(urls: string[]): void;
  scheme(scheme: Scheme): void;
  /** Sends a message of `@compmd/render`'s protocol, for a page frame, once the frame has its level. */
  send(message: unknown): void;
  /**
   * `port`, posting as it does, and counting each change it sends, as `send` does: for `@compmd/render`'s
   * `connectFrame` on the same port, so `settled` counts its renders and splices too.
   */
  counting(port: Port): Port;
  /** Asks the frame to record what passes in it and report it (`observed`), or to stop. */
  observe(on: boolean): void;
  /**
   * Resolves once the frame has settled after everything sent it so far (`settled` with that count), once it is
   * closed or stopped, or after `BUSY_MS`: a frame's code may never say it.
   */
  settled(): Promise<void>;
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
  const listeners: Listeners = { set: [], error: [], recovered: [], link: [], loaded: [], rendered: [], size: [], failed: [], observed: [], refused: [] };
  const emit = <T extends keyof ChildEvents>(type: T, event: ChildEvents[T]) => {
    for (const listener of listeners[type]) listener(event);
  };
  /** What waits for the frame's `ready`; `undefined` once it came. */
  let waiting: (() => void)[] | undefined = [];
  let closed = false;
  /** Whether the parent asked the frame to report: it takes `observed` only then. */
  let observing = false;
  const whenReady = (send: () => void) => {
    if (closed) return;
    if (waiting) waiting.push(send);
    else send();
  };
  /** The messages sent that change what the frame shows, and the count the frame last settled after. */
  let changes = 0;
  let settledAfter = 0;
  /** What waits for the frame to settle after the changes sent. */
  let settling: (() => void)[] = [];
  const settle = () => {
    const waiting = settling;
    settling = [];
    for (const resolve of waiting) resolve();
  };
  /** Sends a sandbox message, or one of `@compmd/render`'s, once the frame is ready. */
  const post = (message: unknown) => {
    whenReady(() => {
      if (isChange(message)) changes += 1;
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
  // A frame whose runtime never loads says nothing at all: without this, its use would wait for it forever.
  const unloaded = options.ping
    ? setTimeout(() => {
        if (waiting) stop({ reason: "unloaded" });
      }, Math.max(options.ping.timeout, LOAD_MS))
    : undefined;
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
      clearTimeout(unloaded);
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
        // As text: strict code may have written anything as the input's name.
        else emit("refused", { input: String(data.input as unknown) });
        break;
      case "error":
        emit("error", {
          reason: data.reason,
          message: data.message,
          ...(data.component === undefined ? {} : { component: data.component }),
          ...(typeof data.use === "string" ? { use: data.use } : {}),
        });
        break;
      case "recovered": {
        const { component, use } = data as { component: unknown; use: unknown };
        if (typeof component === "string" && typeof use === "string") emit("recovered", { component, use });
        break;
      }
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
      case "observed":
        // Unasked, it is a report no one wants, and code of the frame's level may have sent it.
        if (observing && Array.isArray(data.events)) emit("observed", { events: data.events.slice(0, BATCH) });
        break;
      case "image": {
        // Strict code may have written anything as the URL: one that isn't text gets no answer.
        const { url } = data as { url: unknown };
        if (typeof url !== "string") break;
        void (options.image?.(url) ?? Promise.resolve(undefined))
          .catch(() => undefined)
          .then((image) => {
            if (!closed) port.postMessage({ type: "fetched", url, ...(image ? { image } : {}) } satisfies ToFrame);
          });
        break;
      }
      case "settled":
        // For an older change, the frame isn't settled yet: it will say so again for the latest.
        if (data.seen === changes) {
          settledAfter = changes;
          settle();
        }
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
    clearTimeout(unloaded);
    if (pings !== undefined) clearInterval(pings);
    port.removeEventListener("message", listener);
    // A frame that went, or was stopped, holds nothing up.
    settle();
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
    scheme: (scheme) => {
      post({ type: "scheme", scheme } satisfies ToFrame);
    },
    send: (message) => {
      post(message);
    },
    counting: (through) => ({
      postMessage: (message, transfer) => {
        if (isChange(message)) changes += 1;
        through.postMessage(message, transfer);
      },
      addEventListener: (type, listener) => {
        through.addEventListener(type, listener);
      },
      removeEventListener: (type, listener) => {
        through.removeEventListener(type, listener);
      },
      start: () => through.start?.(),
    }),
    observe: (on) => {
      if (on === observing) return;
      observing = on;
      post({ type: "observe", on } satisfies ToFrame);
    },
    settled: () => {
      // Messages waiting for `ready` count once sent: until then the frame isn't settled.
      if (closed || (!waiting && settledAfter === changes)) return Promise.resolve();
      return new Promise<void>((resolve) => {
        const timer = setTimeout(resolve, BUSY_MS);
        settling.push(() => {
          clearTimeout(timer);
          resolve();
        });
      });
    },
    on: (type, listener) => {
      listeners[type].push(listener);
    },
    close,
  };
}
