// The frame's end of the protocol: it believes its port, which its parent gave it, and keeps the order the protocol
// asks for. The level comes first and once: before it, the frame acts on nothing, since its policy isn't narrowed yet.

import { isMessage, SANDBOX_VERSION, TO_FRAME, type CompileRequest, type ComponentCode, type ErrorReason, type FromFrame, type Level, type Part, type Port, type ToFrame } from "./messages.ts";

/** What the frame does with each message; the frame runtime implements it. */
export interface FrameHandlers {
  /** The frame's level and, for a permissive frame, its second policy: applied before this returns. */
  level(level: Level, policy: string | undefined): void;
  /** Loads the components' code; resolves once each is loaded, or shows its fallback. */
  code?(components: ComponentCode[]): Promise<void>;
  part?(part: Part): void;
  styles?(urls: string[]): void;
  /** The port for `host`, which only a permissive frame takes. */
  host?(port: MessagePort): void;
  /** Compiles a source: what only the compile frame does. */
  compile?(request: CompileRequest): Promise<{ code: string; css?: string }>;
}

/** What the frame tells its parent. */
export interface FrameEnd {
  set(input: string, value: unknown): void;
  error(reason: ErrorReason, message: string, component?: string): void;
  link(url: string): void;
  rendered(ms: number): void;
  size(width: number, height: number): void;
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
  const listener = ({ data, ports }: { data: unknown; ports?: readonly MessagePort[] }) => {
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
    }
  };
  port.addEventListener("message", listener);
  port.start?.();
  post({ type: "ready", sandbox: SANDBOX_VERSION });
  return {
    set: (input, value) => {
      post({ type: "set", input, value });
    },
    error: (reason, message, component) => {
      post({ type: "error", reason, message, ...(component === undefined ? {} : { component }) });
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
    close: () => {
      port.removeEventListener("message", listener);
    },
  };
}
