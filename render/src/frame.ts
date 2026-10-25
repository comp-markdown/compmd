// The host and frame protocol: the page renders inside a frame its caller supplies, such as a sandboxed
// iframe, and a port carries trees in and `set` calls out. Both ends take a port their caller made, so a host with its
// own channel to the frame (a sandbox's bootstrap) hands one over, and any bridge fits behind a small adapter.
import type { ComponentError, Mount } from "./contract.ts";
import { createPage, type Page, type PageOptions } from "./page.ts";
import type { Change } from "./stable.ts";
import type { ExpandedRoot } from "./tree.ts";

/** The protocol's version, sent in `ready`; a host checks it, since a host page and its frame may be deployed apart. */
export const PROTOCOL_VERSION = 1;

/** What the protocol needs of a port: a `MessagePort` is one. */
export interface Port {
  postMessage(message: unknown): void;
  addEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  removeEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  start?(): void;
}

/** Messages from the host to the frame. */
export type HostMessage = { type: "render"; tree: ExpandedRoot; change?: Change } | { type: "theme"; tokens: Record<string, string> };

/** Messages from the frame to the host. */
export type FrameMessage =
  | { type: "ready"; version: number }
  | { type: "set"; input: string; value: unknown }
  | { type: "error"; component: string; reason: ComponentError["reason"]; message: string }
  | { type: "rendered"; ms: number };

export interface FrameOptions extends Omit<PageOptions, "onSet" | "onError"> {
  /** Where the page renders, inside the frame. */
  container: HTMLElement;
  /** Host-language components by name; component code stays in the frame, since modules can't be posted. */
  components?: Record<string, Mount>;
}

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : typeof error === "string" ? error : "");

/**
 * Runs the frame's end: renders what the host sends into `options.container`, and tells the host about `set` calls,
 * components that show their fallback, and each render once it is painted.
 *
 * @example
 * // inside the frame
 * addEventListener("message", ({ ports: [port] }) => serveFrame(port, { container: document.body, components }));
 */
export function serveFrame(port: Port, options: FrameOptions): { page: Page; close(): void } {
  const post = (message: FrameMessage) => {
    port.postMessage(message);
  };
  const page = createPage(options.container, {
    ...options,
    onSet: (input, value) => {
      post({ type: "set", input, value });
    },
    onError: ({ component, reason, error }) => {
      post({ type: "error", component, reason, message: messageOf(error) });
    },
  });
  const view = options.container.ownerDocument.defaultView;
  const listener = ({ data }: { data: unknown }) => {
    const message = data as HostMessage;
    if (message.type === "theme") page.theme(message.tokens);
    if (message.type !== "render") return;
    const started = performance.now();
    page.render(message.tree, message.change);
    // Painted: after the next frame, and once the browser has drawn it.
    const painted = () => {
      setTimeout(() => {
        post({ type: "rendered", ms: performance.now() - started });
      }, 0);
    };
    if (view?.requestAnimationFrame) view.requestAnimationFrame(painted);
    else painted();
  };
  port.addEventListener("message", listener);
  port.start?.();
  post({ type: "ready", version: PROTOCOL_VERSION });
  return {
    page,
    close() {
      port.removeEventListener("message", listener);
      page.destroy();
    },
  };
}

type Listeners = { [T in FrameMessage["type"]]: ((message: Extract<FrameMessage, { type: T }>) => void)[] };

export interface FrameConnection {
  /** Renders `tree` in the frame; sent once the frame is ready. */
  render(tree: ExpandedRoot, change?: Change): void;
  theme(tokens: Record<string, string>): void;
  on<T extends FrameMessage["type"]>(type: T, listener: (message: Extract<FrameMessage, { type: T }>) => void): void;
  close(): void;
}

/**
 * Runs the host's end. Messages wait until the frame says it is ready, with this protocol's version; a frame with
 * another version gets nothing, and `on("ready")` tells the host which version it has.
 *
 * @example
 * const { port1, port2 } = new MessageChannel();
 * iframe.contentWindow.postMessage("compmd", "*", [port2]);
 * const frame = connectFrame(port1);
 * frame.on("set", ({ input, value }) => rerender({ [input]: value }));
 * frame.render(tree);
 */
export function connectFrame(port: Port): FrameConnection {
  const listeners: Listeners = { ready: [], set: [], error: [], rendered: [] };
  let waiting: HostMessage[] | undefined = [];
  const send = (message: HostMessage) => {
    if (waiting) waiting.push(message);
    else port.postMessage(message);
  };
  const listener = ({ data }: { data: unknown }) => {
    const message = data as FrameMessage;
    if (message.type === "ready" && message.version === PROTOCOL_VERSION && waiting) {
      for (const queued of waiting) port.postMessage(queued);
      waiting = undefined;
    }
    for (const notify of listeners[message.type] as ((message: FrameMessage) => void)[] | undefined ?? []) notify(message);
  };
  port.addEventListener("message", listener);
  port.start?.();
  return {
    render(tree, change) {
      send({ type: "render", tree, ...(change ? { change } : {}) });
    },
    theme(tokens) {
      send({ type: "theme", tokens });
    },
    on(type, notify) {
      listeners[type].push(notify);
    },
    close() {
      port.removeEventListener("message", listener);
    },
  };
}
