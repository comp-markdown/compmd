// The host and frame protocol: the page renders inside a frame its caller supplies, such as a sandboxed
// iframe, and a port carries trees in and `set` calls out. Both ends take a port their caller made, so a host with its
// own channel to the frame (a sandbox's bootstrap) hands one over, and any bridge fits behind a small adapter.
import type { Mount, RenderError } from "./contract.ts";
import type { Line, Rect } from "./lines.ts";
import { createPage, type Page, type PageOptions } from "./page.ts";
import { reveal, type Source } from "./source.ts";
import type { Change } from "./stable.ts";
import type { ExpandedRoot, Point, Splice } from "./tree.ts";

export type { Line, Rect };

/** The protocol's version, sent in `ready`; a host checks it, since a host page and its frame may be deployed apart. */
export const PROTOCOL_VERSION = 4;

/** How many lines a frame measures for one `lines` message. */
export const LINES_BATCH = 200;

/** What the protocol needs of a port: a `MessagePort` is one. */
export interface Port {
  postMessage(message: unknown): void;
  addEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  removeEventListener(type: "message", listener: (event: { data: unknown }) => void): void;
  start?(): void;
}

/**
 * Messages from the host to the frame. A splice may be its JSON text, as a processor gives it: posted text is copied
 * in one piece, and the frame parses it once. The frame ignores any other message, so a host may send its own on the
 * same port.
 */
export type HostMessage =
  | { type: "render"; tree: ExpandedRoot; change?: Change }
  | { type: "splices"; splices: (Splice | string)[]; change?: Change; replace?: boolean }
  | { type: "theme"; tokens: Record<string, string> }
  /** Scroll to what the source at `offset`, a code-point offset, drew, and mark it briefly (`reveal`). */
  | { type: "reveal"; offset: number }
  /** Which boxes what the source at `offset` drew has: the frame answers with `rects` and the same `id`. */
  | { type: "rects"; id: number; offset: number }
  /** Report the source of what the reader clicks (`pointed`), or stop. */
  | { type: "point"; on: boolean }
  /** Up to `count` lines' places, from line `from`: the frame answers with `lines` and the same `id`. */
  | { type: "lines"; id: number; from: number; count: number };

/** How a host sends splices. */
export interface SpliceOptions {
  /** The first splice is of an empty tree, as after opening a document (again): it replaces every block. */
  replace?: boolean;
}

/** Messages from the frame to the host. */
export type FrameMessage =
  | { type: "ready"; version: number }
  | { type: "set"; input: string; value: unknown }
  | { type: "error"; reason: RenderError["reason"]; component?: string; use?: string; message: string }
  /** A use that failed draws again, by the `use` its `error` named. */
  | { type: "recovered"; component: string; use: string }
  | { type: "rendered"; ms: number }
  /** The answer to `rects` of the same `id`: one box per element, none when nothing came from there. */
  | { type: "rects"; id: number; rects: Rect[] }
  /** The reader clicked what came from `source`, while the host asked with `point`. */
  | { type: "pointed"; source: Source }
  /** The answer to `lines` of the same `id`, in the frame's page: the lines from its `from`, and whether they were the last. */
  | { type: "lines"; id: number; lines: Line[]; done: boolean }
  /** What the frame's page shows may have moved, so its lines may have: as a page's `onLayout`. */
  | { type: "layout" }
  /** The page has drawn (`Page.drawn`) after the first `seen` renders and splices the frame took. */
  | { type: "drawn"; seen: number };

export interface FrameOptions extends Omit<PageOptions, "onSet" | "onError" | "onRecovered"> {
  /** Where the page renders, inside the frame. */
  container: HTMLElement;
  /** Host-language components by name; component code stays in the frame, since modules can't be posted. */
  components?: Record<string, Mount>;
}

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : typeof error === "string" ? error : "");

/** A `splices` message on `page`; one that replaces every block renders the first splice's nodes as a whole tree. */
function applySplices(page: Page, { splices, change, replace }: Extract<HostMessage, { type: "splices" }>): void {
  const parsed = splices.map((splice) => (typeof splice === "string" ? (JSON.parse(splice) as Splice) : splice));
  const [first, ...rest] = parsed;
  if (!replace || !first) {
    page.splice(parsed, change);
    return;
  }
  page.render({ type: "root", children: first.nodes, ...(first.inputs ? { inputs: first.inputs } : {}) }, change);
  if (rest.length > 0) page.splice(rest, change);
}

const rectOf = (element: Element): Rect => {
  const { x, y, width, height } = element.getBoundingClientRect();
  return { x, y, width, height };
};

/** Up to `count` of `page`'s lines from line `from`, and whether there are no more. */
function linesFrom(page: Page, from: number, count: number): { lines: Line[]; done: boolean } {
  const lines: Line[] = [];
  const all = page.lines(from);
  for (let next = all.next(); !next.done; next = all.next()) {
    if (lines.length === count) return { lines, done: false };
    lines.push(next.value);
  }
  return { lines, done: true };
}

/**
 * Runs the frame's end: renders what the host sends into `options.container`, and tells the host about `set` calls,
 * components that show their fallback, and each render once it is painted. It scrolls to and marks what a source
 * offset drew (`reveal`), gives its boxes (`rects`), and, once asked (`point`), the source of what the reader clicks.
 * It gives each line's place (`lines`, a batch at a time), says when they may have moved (`layout`), and when the
 * page has drawn after the renders and splices it took (`drawn`).
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
    onError: (error) => {
      if ("component" in error) post({ type: "error", reason: error.reason, component: error.component, ...(error.use === undefined ? {} : { use: error.use }), message: messageOf(error.error) });
      else post({ type: "error", reason: error.reason, message: error.message });
    },
    onRecovered: ({ component, use }) => {
      post({ type: "recovered", component, use });
    },
    onLayout: () => {
      options.onLayout?.();
      post({ type: "layout" });
    },
  });
  const view = options.container.ownerDocument.defaultView;
  let pointing = false;
  /** The renders and splices taken so far: `drawn` says for how many the page has drawn. */
  let seen = 0;
  const drawn = () => {
    const after = seen;
    void page.drawn().then(() => {
      if (after === seen) post({ type: "drawn", seen });
    });
  };
  // Captured, so a component that stops a click's propagation still has it reported.
  const pointed = (event: Event) => {
    // A click's target is the node clicked, inside the container.
    const target = event.target as Node | null;
    const source = pointing && target ? page.sourceOf(target) : undefined;
    if (source) post({ type: "pointed", source });
  };
  options.container.addEventListener("click", pointed, { capture: true });
  const listener = ({ data }: { data: unknown }) => {
    const message = data as HostMessage;
    const started = performance.now();
    switch (message.type) {
      case "theme":
        page.theme(message.tokens);
        return;
      case "reveal":
        reveal(page.elementsAt(message.offset));
        return;
      case "rects":
        post({ type: "rects", id: message.id, rects: page.elementsAt(message.offset).map(rectOf) });
        return;
      case "point":
        pointing = message.on;
        return;
      case "lines":
        post({ type: "lines", id: message.id, ...linesFrom(page, message.from, Math.min(message.count, LINES_BATCH)) });
        return;
      case "render":
        page.render(message.tree, message.change);
        seen += 1;
        drawn();
        break;
      case "splices":
        applySplices(page, message);
        seen += 1;
        drawn();
        break;
      default:
        // A host may share the port with messages of its own, such as an editor's components to compile.
        return;
    }
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
      options.container.removeEventListener("click", pointed, { capture: true });
      page.destroy();
    },
  };
}

type Listeners = { [T in FrameMessage["type"]]: ((message: Extract<FrameMessage, { type: T }>) => void)[] };

export interface FrameConnection {
  /** Renders `tree` in the frame; sent once the frame is ready. */
  render(tree: ExpandedRoot, change?: Change): void;
  /** Applies `splices` in the frame, as `Page.splice` does; each may be a processor's JSON text, passed on unread. */
  splice(splices: readonly (Splice | string)[], change?: Change, options?: SpliceOptions): void;
  theme(tokens: Record<string, string>): void;
  /** Scrolls the frame's page to what the source at `offset`, a code-point offset, drew, and marks it briefly. */
  reveal(offset: number): void;
  /**
   * The boxes of what the source at `offset` drew, one per element, in the frame's viewport: a host adds where the
   * frame is in its own. None when nothing came from there, or once the connection is closed.
   */
  rectsAt(offset: number): Promise<Rect[]>;
  /** Asks the frame to report the source of what the reader clicks, as `pointed`, or to stop. */
  point(on: boolean): void;
  /**
   * Each line of the frame's page that draws something, from line `from`, with its boxes in the frame's page, as a
   * page's `lines`: the frame measures a batch at a time, as the loop asks for them, and none once the loop stops or
   * the connection is closed. `on("layout")` says when to ask again.
   *
   * @example
   * frame.on("layout", async () => {
   *   for await (const { line, rects } of frame.lines()) place(line, rects);
   * });
   */
  lines(from?: number): AsyncGenerator<Line>;
  /**
   * Listens for the frame's messages. A frame's code may be untrusted: `pointed` carries only a well-formed source,
   * `rects` and `lines` only boxes that are numbers and lines that are whole numbers from 1, and other messages are as
   * the frame sent them.
   */
  on<T extends FrameMessage["type"]>(type: T, listener: (message: Extract<FrameMessage, { type: T }>) => void): void;
  close(): void;
}

const isCount = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value);

/** `value` as a point, if it is one: a frame's code may have sent anything. */
function pointOf(value: unknown): Point | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const { line, column, offset } = value as Record<string, unknown>;
  if (!isCount(line) || !isCount(column) || (offset !== undefined && !isCount(offset))) return undefined;
  return { line, column, ...(offset === undefined ? {} : { offset }) };
}

function sourceOf(value: unknown): Source | undefined {
  if (typeof value !== "object" || value === null) return undefined;
  const { start, end } = value as Record<string, unknown>;
  const [from, to] = [pointOf(start), pointOf(end)];
  return from && to ? { start: from, end: to } : undefined;
}

/** The boxes of a `rects` answer that are boxes. */
function rectsOf(value: unknown): Rect[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((rect: unknown) => {
    if (typeof rect !== "object" || rect === null) return [];
    const { x, y, width, height } = rect as Record<string, unknown>;
    return isCount(x) && isCount(y) && isCount(width) && isCount(height) ? [{ x, y, width, height }] : [];
  });
}

/** The lines of a `lines` answer that are lines: a whole number from 1, with its boxes. */
function linesOf(value: unknown): Line[] {
  if (!Array.isArray(value)) return [];
  return value.flatMap((line: unknown) => {
    if (typeof line !== "object" || line === null) return [];
    const { line: number, rects } = line as Record<string, unknown>;
    return Number.isInteger(number) && (number as number) >= 1 ? [{ line: number as number, rects: rectsOf(rects) }] : [];
  });
}

type Answer = Extract<FrameMessage, { type: "lines" }>;

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
  const listeners: Listeners = { ready: [], set: [], error: [], recovered: [], rendered: [], rects: [], pointed: [], lines: [], layout: [], drawn: [] };
  let waiting: HostMessage[] | undefined = [];
  const send = (message: HostMessage) => {
    if (waiting) waiting.push(message);
    else port.postMessage(message);
  };
  /** The `rects` asked for and not answered yet, by id. */
  const asked = new Map<number, (rects: Rect[]) => void>();
  /** The `lines` asked for and not answered yet, by id. */
  const askedLines = new Map<number, (answer: Answer) => void>();
  let lastAsked = 0;
  const notify = (message: FrameMessage) => {
    for (const listen of listeners[message.type] as ((message: FrameMessage) => void)[] | undefined ?? []) listen(message);
  };
  const listener = ({ data }: { data: unknown }) => {
    const message = data as FrameMessage;
    if (message.type === "ready" && message.version === PROTOCOL_VERSION && waiting) {
      for (const queued of waiting) port.postMessage(queued);
      waiting = undefined;
    }
    if (message.type === "rects") {
      const answer = asked.get(message.id);
      asked.delete(message.id);
      const rects = rectsOf(message.rects);
      answer?.(rects);
      notify({ type: "rects", id: message.id, rects });
      return;
    }
    if (message.type === "pointed") {
      const source = sourceOf(message.source);
      if (source) notify({ type: "pointed", source });
      return;
    }
    if (message.type === "drawn") {
      if (Number.isInteger(message.seen)) notify({ type: "drawn", seen: message.seen });
      return;
    }
    if (message.type === "lines") {
      // Anything but `false` ends the loop, so a frame that leaves it out can't keep it going.
      const done: unknown = message.done;
      const answer: Answer = { type: "lines", id: message.id, lines: linesOf(message.lines), done: done !== false };
      askedLines.get(message.id)?.(answer);
      askedLines.delete(message.id);
      notify(answer);
      return;
    }
    notify(message);
  };
  /** Asks the frame for a batch of lines from `from`; an empty last batch once the connection is closed. */
  const batch = (from: number): Promise<Answer> => {
    const id = ++lastAsked;
    return new Promise((resolve) => {
      askedLines.set(id, resolve);
      send({ type: "lines", id, from, count: LINES_BATCH });
    });
  };
  port.addEventListener("message", listener);
  port.start?.();
  return {
    reveal(offset) {
      send({ type: "reveal", offset });
    },
    rectsAt(offset) {
      const id = ++lastAsked;
      return new Promise((resolve) => {
        asked.set(id, resolve);
        send({ type: "rects", id, offset });
      });
    },
    point(on) {
      send({ type: "point", on });
    },
    async *lines(from = 1) {
      for (let next = from; ; ) {
        const { lines, done } = await batch(next);
        // Only lines after those given: a frame's code could send the same again, and keep the loop going.
        const after = lines.filter(({ line }) => line >= next);
        yield* after;
        const last = after.at(-1);
        if (done || !last) return;
        next = last.line + 1;
      }
    },
    render(tree, change) {
      send({ type: "render", tree, ...(change ? { change } : {}) });
    },
    splice(splices, change, options = {}) {
      send({ type: "splices", splices: [...splices], ...(change ? { change } : {}), ...(options.replace ? { replace: true } : {}) });
    },
    theme(tokens) {
      send({ type: "theme", tokens });
    },
    on(type, notify) {
      listeners[type].push(notify);
    },
    close() {
      port.removeEventListener("message", listener);
      for (const answer of asked.values()) answer([]);
      asked.clear();
      for (const [id, answer] of askedLines) answer({ type: "lines", id, lines: [], done: true });
      askedLines.clear();
    },
  };
}
