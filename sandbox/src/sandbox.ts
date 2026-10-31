// The sandbox's default for a whole document (docs/sandbox.md, Q1, Q7): the host chooses where the page renders. In
// frame mode the page is one strict frame per document; in host mode the page renders in the host's page, trusted
// components inline, every other use in a frame of its level. Either way the host gives trees and gets `set`, links
// and errors, and never writes a frame or a policy itself.

import type { HostComponent } from "@compmd/processor";
import { createPage, type Change, type ExpandedInput, type ExpandedRoot, type Page, type Splice } from "@compmd/render";
import { connectFrame, type FrameConnection, type SpliceOptions } from "@compmd/render/frame";
import { HostRuntime, type HostRuntimeOptions } from "./host/runtime.ts";
import { codeFor } from "./mounts.ts";
import { HELLO, SANDBOX_VERSION, type ErrorReason } from "./protocol/messages.ts";
import { connectChild, type ChildEnd } from "./protocol/parent-end.ts";

export interface SandboxOptions extends HostRuntimeOptions {
  /** Where the page renders: in a strict frame per document (the default), or in the host's page. */
  page?: "frame" | "host";
}

export interface ViewEvents {
  set: { input: string; value: unknown };
  /** A reader clicked a link in the page; the host shows its URL before it opens (spec §15 rule 4). */
  link: { url: string };
  error: { component: string; reason: ErrorReason; message: string };
}

/** How a host sends splices. */
export interface ViewSpliceOptions extends SpliceOptions {
  /**
   * The document's inputs after the splices, which its page may set: what a processor's `ChangesJson` gives beside
   * splices it sends as text. Splices as objects carry their own.
   */
  inputs?: Readonly<Record<string, ExpandedInput>>;
}

/** One document in its sandbox. */
export interface View {
  /** Renders `tree`, as `@compmd/render`'s page does. */
  render(tree: ExpandedRoot, change?: Change): void;
  /** Applies `splices`, each of them maybe a processor's JSON text, as `@compmd/render`'s page does. */
  splice(splices: readonly (Splice | string)[], change?: Change, options?: ViewSpliceOptions): void;
  /** The document's components are now `components`: resolves once each is compiled, or has failed. */
  components(components: readonly HostComponent[]): Promise<void>;
  theme(tokens: Record<string, string>): void;
  on<T extends keyof ViewEvents>(type: T, listener: (event: ViewEvents[T]) => void): void;
  destroy(): void;
}

export interface Sandbox {
  /** Opens a document in `container`, which the page fills, with its components. */
  open(container: HTMLElement, options: { components: readonly HostComponent[] }): View;
}

type Listeners = { [T in keyof ViewEvents]: ((event: ViewEvents[T]) => void)[] };

/**
 * A sandbox for a host's documents.
 *
 * @example
 * const sandbox = createSandbox({ base: "/compmd/", page: "host", trusted: { stars: react(Stars) } });
 * const view = sandbox.open(container, { components: processed.hostComponents });
 * view.render(processed.expanded);
 * view.on("set", ({ input, value }) => rerender({ [input]: value }));
 */
export function createSandbox(options: SandboxOptions): Sandbox {
  return {
    open(container, { components }) {
      const listeners: Listeners = { set: [], link: [], error: [] };
      const emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => {
        for (const listener of listeners[type]) listener(event);
      };
      const runtime = new HostRuntime(options, {
        set: (input, value) => {
          emit("set", { input, value });
        },
        link: (url) => {
          emit("link", { url });
        },
        error: (component, reason, message) => {
          emit("error", { component, reason, message });
        },
      });
      const page = options.page === "host" ? hostPage(container, runtime, emit) : framePage(container, runtime, options, emit);
      const ready = runtime.set(components).then(() => {
        page.code();
      });
      return {
        render: (tree, change) => {
          void ready.then(() => {
            page.render(tree, change);
          });
        },
        splice: (splices, change, spliceOptions) => {
          void ready.then(() => {
            page.splice(splices, change, spliceOptions);
          });
        },
        components: async (next) => {
          await runtime.set(next);
          page.code();
        },
        theme: (tokens) => {
          runtime.theme(tokens);
          page.theme(tokens);
        },
        on: (type, listener) => {
          listeners[type].push(listener);
        },
        destroy: () => {
          page.destroy();
          runtime.destroy();
        },
      };
    },
  };
}

/** Where a view's page renders: what each mode does with trees, code and the theme. */
interface PageEnd {
  render(tree: ExpandedRoot, change?: Change): void;
  splice(splices: readonly (Splice | string)[], change?: Change, options?: ViewSpliceOptions): void;
  /** The runtime's code changed. */
  code(): void;
  theme(tokens: Record<string, string>): void;
  destroy(): void;
}

type Emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => void;

/** Host mode: the page in the host's page, with the runtime's components. */
function hostPage(container: HTMLElement, runtime: HostRuntime, emit: Emit): PageEnd {
  const page: Page = createPage(container, {
    components: runtime.mounts,
    onSet: (input, value) => {
      emit("set", { input, value });
    },
    onError: (error) => {
      if ("component" in error) emit("error", { component: error.component, reason: error.reason, message: error.error instanceof Error ? error.error.message : "" });
    },
  });
  // A link in the page is the host's own to open, as any of its links; links in frames come as `link`.
  return {
    render: (tree, change) => {
      page.render(tree, change);
    },
    splice: (splices, change) => {
      page.splice(splices.map((splice) => (typeof splice === "string" ? (JSON.parse(splice) as Splice) : splice)), change);
    },
    code: () => undefined,
    theme: (tokens) => {
      page.theme(tokens);
    },
    destroy: () => {
      page.destroy();
    },
  };
}

/** Frame mode: the page in one strict frame, which renders it through `@compmd/render`'s frame protocol. */
function framePage(container: HTMLElement, runtime: HostRuntime, options: SandboxOptions, emit: Emit): PageEnd {
  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts");
  iframe.setAttribute("allow", "");
  iframe.title = "The rendered page";
  iframe.className = "compmd-page-frame";
  iframe.style.cssText = "border: 0; width: 100%; height: 100%; display: block;";
  /** The document's inputs, which its page may set, from the last tree or splice. */
  let inputs: Readonly<Record<string, ExpandedInput>> = {};
  const { port1, port2 } = new MessageChannel();
  const child: ChildEnd = connectChild(port1, { level: "strict", binds: (input) => input in inputs, ping: { every: 1000, timeout: options.timeout ?? 5000 } });
  // `@compmd/render`'s protocol on the same port: its messages wait for the page's renderer, which starts after the level.
  const frame: FrameConnection = connectFrame(port1);
  child.on("set", (event) => {
    emit("set", event);
  });
  child.on("link", (event) => {
    emit("link", event);
  });
  child.on("error", ({ reason, component, message }) => {
    emit("error", { component: component ?? "", reason, message });
  });
  child.on("failed", (failure) => {
    emit("error", { component: "", reason: "threw", message: failure.reason === "hung" ? "the page's frame stopped answering" : `the page's frame is of protocol version ${String(failure.version)}` });
  });
  if (options.stylesheets?.length) child.styles([...options.stylesheets]);
  iframe.addEventListener("load", () => iframe.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]), { once: true });
  iframe.src = new URL("strict.html", options.base).href;
  container.append(iframe);
  return {
    render: (tree, change) => {
      inputs = tree.inputs ?? {};
      frame.render(tree, change);
    },
    splice: (splices, change, spliceOptions = {}) => {
      for (const splice of splices) if (typeof splice !== "string" && splice.inputs) inputs = splice.inputs;
      if (spliceOptions.inputs) inputs = spliceOptions.inputs;
      frame.splice(splices, change, spliceOptions.replace ? { replace: true } : {});
    },
    code: () => {
      child.code(codeFor(runtime.code, "strict"));
    },
    theme: (tokens) => {
      frame.theme(tokens);
    },
    destroy: () => {
      frame.close();
      child.close();
      iframe.remove();
    },
  };
}
