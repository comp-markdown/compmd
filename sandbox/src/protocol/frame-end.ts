// The frame's end of the protocol: it believes its port, which its parent gave it, and keeps the order the protocol
// asks for. The level comes first and once: before it, the frame acts on nothing, since its policy isn't narrowed yet.

import { isChange, isMessage, SANDBOX_VERSION, TO_FRAME, type CompileRequest, type ComponentCode, type ErrorReason, type FromFrame, type Level, type Part, type Port, type Scheme, type ToFrame } from "./messages.ts";
import type { Observed } from "./observed.ts";

/** What the frame does with each message; the frame runtime implements it. */
export interface FrameHandlers {
  /** The frame's level and, for a permissive frame, its second policy: applied before this returns. */
  level(level: Level, policy: string | undefined): void;
  /** Loads the components' code; resolves once each is loaded, or shows its fallback. */
  code?(components: ComponentCode[]): Promise<void>;
  part?(part: Part): void;
  styles?(urls: string[]): void;
  /** The page's colour scheme, which the frame's document takes. */
  scheme?(scheme: Scheme): void;
  /** The port for `host`, which only a permissive frame takes. */
  host?(port: MessagePort): void;
  /** Compiles a source: what only the compile frame does. */
  compile?(request: CompileRequest): Promise<{ code: string; css?: string }>;
  /** The parent asks the frame to record what passes in it and report it, or to stop. */
  observe?(on: boolean): void;
  /**
   * A message that changes what the frame shows came (`CHANGES`), the sandbox's or `@compmd/render`'s, after its own
   * handler: the frame settles again.
   */
  changed?(): void;
}

/** What the frame tells its parent. */
export interface FrameEnd {
  set(input: string, value: unknown): void;
  /** A component shows its fallback; `use` names the use, for a later `recovered`. */
  error(reason: ErrorReason, message: string, component?: string, use?: string): void;
  /** The use `use` of `component`, which failed, draws again. */
  recovered(component: string, use: string): void;
  link(url: string): void;
  rendered(ms: number): void;
  size(width: number, height: number): void;
  /** Reports what the frame's observer recorded, once the parent asked. */
  observed(events: Observed[]): void;
  /** Asks for an image of the frame's page by its `src`: its file, or `undefined` when the parent gives none. */
  image(url: string): Promise<Blob | undefined>;
  /** How many messages that change what the frame shows it has taken. */
  readonly seen: number;
  /** The frame has settled after the first `seen` of them. */
  settled(seen: number): void;
  close(): void;
}

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/**
 * Serves the frame's end on `port`, the one its parent gave it: says it is ready, then takes the parent's messages in
 * the protocol's order.
 *
 * @example
 * const parent = serveParent(port, { level: applyPolicy, code: load, part: render });
 * parent.set("region", "eu");
 */
export function serveParent(port: Port, handlers: FrameHandlers): FrameEnd {
  const post = (message: FromFrame) => {
    port.postMessage(message);
  };
  let level: Level | undefined;
  /** The images asked for and not answered yet, by URL. */
  const images = new Map<string, ((image: Blob | undefined) => void)[]>();
  let seen = 0;
  /** Counts a change, `@compmd/render`'s too, which its own listener on the port handles after this one. */
  const counting = ({ data }: { data: unknown }) => {
    if (level === undefined || !isChange(data)) return;
    seen += 1;
    // After the listeners of this message, `@compmd/render`'s among them, have run.
    queueMicrotask(() => handlers.changed?.());
  };
  const listener = ({ data, ports }: { data: unknown; ports?: readonly MessagePort[] }) => {
    counting({ data });
    if (!isMessage<ToFrame>(data, TO_FRAME)) return;
    if (data.type === "level") {
      if (level !== undefined) return;
      handlers.level(data.level, data.policy);
      level = data.level;
      return;
    }
    if (level === undefined) return;
    switch (data.type) {
      case "ping":
        post({ type: "pong", n: data.n });
        break;
      case "code":
        void handlers.code?.(data.components).then(() => {
          post({ type: "loaded" });
        });
        break;
      case "part":
        handlers.part?.(data.part);
        break;
      case "styles":
        handlers.styles?.(data.urls);
        break;
      case "scheme":
        handlers.scheme?.(data.scheme === "dark" ? "dark" : "light");
        break;
      case "host": {
        const [hostPort] = ports ?? [];
        if (level === "permissive" && hostPort) handlers.host?.(hostPort);
        break;
      }
      case "compile": {
        const { name, language, runtime, code } = data;
        const request: CompileRequest = { name, language, code, ...(runtime === undefined ? {} : { runtime }) };
        handlers.compile?.(request).then(
          (compiled) => {
            post({ type: "compiled", name: request.name, ...compiled });
          },
          (error: unknown) => {
            post({ type: "failed", name: request.name, message: messageOf(error) });
          },
        );
        break;
      }
      case "observe":
        handlers.observe?.(data.on);
        break;
      case "fetched": {
        const answered = images.get(data.url) ?? [];
        images.delete(data.url);
        for (const resolve of answered) resolve(data.image instanceof Blob ? data.image : undefined);
        break;
      }
    }
  };
  port.addEventListener("message", listener);
  port.start?.();
  post({ type: "ready", sandbox: SANDBOX_VERSION });
  return {
    set: (input, value) => {
      post({ type: "set", input, value });
    },
    error: (reason, message, component, use) => {
      post({ type: "error", reason, message, ...(component === undefined ? {} : { component }), ...(use === undefined ? {} : { use }) });
    },
    recovered: (component, use) => {
      post({ type: "recovered", component, use });
    },
    link: (url) => {
      post({ type: "link", url });
    },
    rendered: (ms) => {
      post({ type: "rendered", ms });
    },
    size: (width, height) => {
      post({ type: "size", width, height });
    },
    observed: (events) => {
      post({ type: "observed", events });
    },
    image: (url) =>
      new Promise((resolve) => {
        const waiting = images.get(url);
        if (waiting) {
          waiting.push(resolve);
          return;
        }
        images.set(url, [resolve]);
        post({ type: "image", url });
      }),
    get seen() {
      return seen;
    },
    settled: (count) => {
      post({ type: "settled", seen: count });
    },
    close: () => {
      port.removeEventListener("message", listener);
    },
  };
}
