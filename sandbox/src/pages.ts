// Where a view's page renders (docs/running-components-in-the-sandbox.md, Q1, Q7): in the host's page, with the
// runtime's components, or in one strict frame, which renders it through `@compmd/render`'s frame protocol.

import { createPage, reveal, type Change, type ExpandedInput, type ExpandedRoot, type Line, type Page, type Splice } from "@compmd/render";
import { connectFrame, type FrameConnection } from "@compmd/render/frame";
import { framesHighlight, framesMath } from "./from-frames.ts";
import { frameUrl, type HostRuntime } from "./host/runtime.ts";
import { PageImages } from "./images.ts";
import { codeFor } from "./mounts.ts";
import type { Observer } from "./observer.ts";
import { diagnosticOf, HELLO, SANDBOX_VERSION, type ErrorReason, type Scheme } from "./protocol/messages.ts";
import { connectChild, failureText, type ChildEnd } from "./protocol/parent-end.ts";
import type { SandboxOptions, ViewEvents, ViewSpliceOptions } from "./sandbox.ts";

/** The `error` event for `component`, with its diagnostic, and its use when it has one. */
export const errorEvent = (component: string, reason: ErrorReason, message: string, use?: string): ViewEvents["error"] => {
  const code = diagnosticOf(reason);
  return { component, reason, ...(code ? { code } : {}), message, ...(use === undefined ? {} : { use }) };
};

/** Where a view's page renders: what each mode does with trees, code and the theme. */
export interface PageEnd {
  render(tree: ExpandedRoot, change?: Change): void;
  splice(splices: readonly (Splice | string)[], change?: Change, options?: ViewSpliceOptions): void;
  /** The runtime's code changed. */
  code(): void;
  theme(tokens: Record<string, string>): void;
  styles(urls: readonly string[]): void;
  /** The page's colour scheme, for the page's own frame. */
  scheme(scheme: Scheme): void;
  /** The host switched its own theme: a page in the host's page reads its scheme again, for its trusted components. */
  reread(): void;
  reveal(offset: number): void;
  rectsAt(offset: number): Promise<DOMRect[]>;
  /** Reports the source of what the reader clicks, as `pointed`, from now on. */
  point(): void;
  /** Each line's place, from the container's content's top left corner: at once in the host's page, from a frame later. */
  lines(from?: number): Iterable<Line> | AsyncIterable<Line>;
  /** What renders in the host's page has drawn: its components mounted, their `busy` drawings settled. */
  drawn(): Promise<void>;
  /** The frames the host's page made for the page, each of which says when it has settled. */
  frames(): Iterable<{ settled(): Promise<void> }>;
  destroy(): void;
}

/** Tells the view's listeners of an event. */
export type Emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => void;

/** Host mode: the page in the host's page, with the runtime's components. */
export function hostPage(container: HTMLElement, runtime: HostRuntime, options: SandboxOptions, observer: Observer, emit: Emit): PageEnd {
  const page: Page = createPage(container, {
    components: runtime.mounts,
    // The frames' own Temml and Shiki, from their files, so math and code look the same in both modes, and the app
    // ships one copy of each, which this page and its frames share.
    math: framesMath(options.base),
    highlight: framesHighlight(options.base),
    // The trusted components in this page get `host` as the runtime gives it; framed ones get theirs from their frame.
    ...(runtime.host ? { host: runtime.host } : {}),
    onSet: (input, value) => {
      emit("set", { input, value });
    },
    onError: (error) => {
      if (!("component" in error)) return;
      const message = error.error instanceof Error ? error.error.message : "";
      observer.fallback(error.component, error.reason, message);
      emit("error", errorEvent(error.component, error.reason, message, error.use));
    },
    onRecovered: (recovered) => {
      emit("recovered", recovered);
    },
    onLayout: () => {
      emit("layout", {});
    },
  });
  // Captured, so a component that stops a click's propagation still has it reported, as in a page's frame.
  const pointed = (event: Event) => {
    const source = page.sourceOf(event.target as Node);
    if (source) emit("pointed", { source });
  };
  /** Reports a render once it is painted: after the next frame, and once the browser has drawn it. */
  const painted = (started: number) => {
    requestAnimationFrame(() => {
      setTimeout(() => {
        emit("rendered", { ms: performance.now() - started });
      }, 0);
    });
  };
  // A link in the page is the host's own to open, as any of its links; links in frames come as `link`.
  return {
    render: (tree, change) => {
      const started = performance.now();
      page.render(tree, change);
      painted(started);
    },
    splice: (splices, change) => {
      const started = performance.now();
      page.splice(splices.map((splice) => (typeof splice === "string" ? (JSON.parse(splice) as Splice) : splice)), change);
      painted(started);
    },
    // The trusted components are the host's, loaded already; each frame loads its own as it mounts.
    code: () => {
      emit("loaded", {});
    },
    theme: (tokens) => {
      page.theme(tokens);
    },
    // The host's page has its own stylesheets and scheme; the runtime gives these to its frames.
    styles: () => undefined,
    scheme: () => undefined,
    reread: () => {
      page.scheme();
    },
    reveal: (offset) => {
      reveal(page.elementsAt(offset));
    },
    rectsAt: (offset) => Promise.resolve(page.elementsAt(offset).map((element) => element.getBoundingClientRect())),
    point: () => {
      container.addEventListener("click", pointed, { capture: true });
    },
    // The page's root is the container: its lines are where they should be.
    lines: (from) => page.lines(from),
    drawn: () => page.drawn(),
    frames: () => runtime.frames(),
    destroy: () => {
      container.removeEventListener("click", pointed, { capture: true });
      page.destroy();
    },
  };
}

/** Frame mode: the page in one strict frame, which renders it through `@compmd/render`'s frame protocol. */
export function framePage(container: HTMLElement, runtime: HostRuntime, options: SandboxOptions, observer: Observer, emit: Emit): PageEnd {
  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts");
  iframe.setAttribute("allow", "");
  iframe.title = "The rendered page";
  iframe.className = "compmd-page-frame";
  iframe.style.cssText = "border: 0; width: 100%; height: 100%; display: block;";
  /** The document's inputs, which its page may set, from the last tree or splice. */
  let inputs: Readonly<Record<string, ExpandedInput>> = {};
  const images = new PageImages(fetch, document.baseURI);
  const { port1, port2 } = new MessageChannel();
  const id = observer.frame("strict", { kind: "page" });
  const port = observer.port(id, port1);
  const child: ChildEnd = connectChild(port, {
    level: "strict",
    binds: (input) => input in inputs,
    ping: { every: 1000, timeout: options.timeout ?? 5000 },
    image: (url) => images.file(url),
  });
  observer.connected(id, child);
  child.on("refused", ({ input }) => {
    observer.refused(id, input);
  });
  // `@compmd/render`'s protocol on the same port: its messages wait for the page's renderer, which starts after the level.
  const frame: FrameConnection = connectFrame(child.counting(port));
  child.on("set", (event) => {
    emit("set", event);
  });
  child.on("link", (event) => {
    emit("link", event);
  });
  child.on("error", ({ reason, component, message, use }) => {
    emit("error", errorEvent(component ?? "", reason, message, use));
  });
  child.on("recovered", (event) => {
    emit("recovered", event);
  });
  child.on("rendered", (event) => {
    emit("rendered", event);
  });
  child.on("loaded", (event) => {
    emit("loaded", event);
  });
  frame.on("pointed", ({ source }) => {
    emit("pointed", { source });
  });
  frame.on("layout", () => {
    emit("layout", {});
  });
  const url = frameUrl(options.base, "strict.html");
  child.on("failed", (failure) => {
    observer.gone(id, failure.reason);
    const message = `the page's frame ${failureText(failure, url)}`;
    // The host's page says why too: a frame that never loaded has nothing of its own in the console.
    if (failure.reason === "unloaded") console.error(`CompMD: ${message}`);
    emit("error", errorEvent("", failure.reason === "unloaded" ? "failed" : "threw", message));
  });
  if (options.stylesheets?.length) child.styles([...options.stylesheets]);
  iframe.addEventListener("load", () => iframe.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]), { once: true });
  iframe.src = url;
  container.append(iframe);
  return {
    render: (tree, change) => {
      inputs = tree.inputs ?? {};
      images.rendered(tree);
      frame.render(tree, change);
    },
    splice: (splices, change, spliceOptions = {}) => {
      for (const splice of splices) if (typeof splice !== "string" && splice.inputs) inputs = splice.inputs;
      if (spliceOptions.inputs) inputs = spliceOptions.inputs;
      images.spliced(splices);
      frame.splice(splices, change, spliceOptions.replace ? { replace: true } : {});
    },
    code: () => {
      child.code(codeFor(runtime.code, "strict"));
    },
    theme: (tokens) => {
      frame.theme(tokens);
    },
    styles: (urls) => {
      child.styles([...urls]);
    },
    scheme: (scheme) => {
      child.scheme(scheme);
    },
    // The page's frame takes its scheme from `scheme`.
    reread: () => undefined,
    reveal: (offset) => {
      frame.reveal(offset);
    },
    // The frame's boxes are in its viewport: where the frame's content starts in the host's is added.
    rectsAt: async (offset) => {
      const rects = await frame.rectsAt(offset);
      const { x, y } = iframe.getBoundingClientRect();
      return rects.map((rect) => new DOMRect(rect.x + x + iframe.clientLeft, rect.y + y + iframe.clientTop, rect.width, rect.height));
    },
    point: () => {
      frame.point(true);
    },
    // The frame's lines are in its page: where the frame's content starts in the container's is added.
    lines: async function* (from) {
      for await (const { line, rects } of frame.lines(from)) {
        const at = iframe.getBoundingClientRect();
        const around = container.getBoundingClientRect();
        const dx = at.x - around.x + iframe.clientLeft + container.scrollLeft;
        const dy = at.y - around.y + iframe.clientTop + container.scrollTop;
        yield { line, rects: rects.map((rect) => ({ ...rect, x: rect.x + dx, y: rect.y + dy })) };
      }
    },
    // Nothing renders here: the page's frame settles after all the host sent it, its own frames with it.
    drawn: () => Promise.resolve(),
    frames: () => [child],
    destroy: () => {
      frame.close();
      child.close();
      iframe.remove();
      observer.gone(id, "removed");
    },
  };
}
