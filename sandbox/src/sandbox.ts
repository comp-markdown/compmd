// The sandbox's default for a whole document (docs/running-components-in-the-sandbox.md, Q1, Q7): the host chooses
// where the page renders. In frame mode the page is one strict frame per document; in host mode the page renders in
// the host's page, trusted components inline, every other use in a frame of its level. Either way the host gives
// trees and gets `set`, links and errors, and never writes a frame or a policy itself.

import type { HostComponent } from "@compmd/processor";
import { createPage, schemeOf, type Change, type ExpandedInput, type ExpandedRoot, type Page, type Splice } from "@compmd/render";
import { connectFrame, type FrameConnection, type SpliceOptions } from "@compmd/render/frame";
import { frameUrl, HostRuntime, type HostRuntimeOptions } from "./host/runtime.ts";
import { codeFor } from "./mounts.ts";
import { Log, reportedBy, type Observation, type ObservedEvent } from "./observation.ts";
import { Observer } from "./observer.ts";
import { diagnosticOf, HELLO, SANDBOX_VERSION, type ErrorReason, type SandboxDiagnostic, type Scheme } from "./protocol/messages.ts";
import { connectChild, type ChildEnd } from "./protocol/parent-end.ts";

export interface SandboxOptions extends HostRuntimeOptions {
  /** Where the page renders: in a strict frame per document (the default), or in the host's page. */
  page?: "frame" | "host";
}

export interface ViewEvents {
  set: { input: string; value: unknown };
  /** A reader clicked a link in the page; the host shows its URL before it opens (spec §15 rule 4). */
  link: { url: string };
  /** A component shows its fallback, why, and the diagnostic it is when it is one (`diagnosticOf`). */
  error: { component: string; reason: ErrorReason; code?: SandboxDiagnostic; message: string };
  /** A render or splice is painted, `ms` after it started. */
  rendered: { ms: number };
  /** The components last given are loaded in the page, or show their fallbacks: a host may show the page from now. */
  loaded: Record<string, never>;
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
  /**
   * The host switched its own theme (a class on its page, not a media query): the page's colour scheme is read again,
   * and its frames and components follow it. It is read at each render too, when the reader's preference changes, and
   * when printing starts and ends.
   */
  scheme(): void;
  /** The host's stylesheets for the page and its frames, in place of `stylesheets`; served from the frames' origin. */
  styles(urls: readonly string[]): void;
  /** Listens for `type`; a listener added as soon as `open` returns hears everything, opening's errors too. */
  on<T extends keyof ViewEvents>(type: T, listener: (event: ViewEvents[T]) => void): void;
  /**
   * Observes the view (docs/devtools.md, decisions 5 and 6), for developer tools: its frames, down to those its frames
   * make, the messages on their ports, the components mounted in each window, with their times, and their fallbacks.
   * `listener` gets each event as it comes; what a frame reported is marked with `by`. Only the host's page can turn
   * this on: nothing a document or a frame sends does. The sandbox observes while an observation listens.
   *
   * @example
   * const observation = view.observe((events) => { tools.update(observation.events); });
   * observation.stop();
   */
  observe(listener: (events: readonly ObservedEvent[]) => void): Observation;
  destroy(): void;
}

export interface Sandbox {
  /** Opens a document in `container`, which the page fills, with its components. */
  open(container: HTMLElement, options: { components: readonly HostComponent[] }): View;
}

/** The `error` event for `component`, with its diagnostic. */
const errorEvent = (component: string, reason: ErrorReason, message: string): ViewEvents["error"] => {
  const code = diagnosticOf(reason);
  return code ? { component, reason, code, message } : { component, reason, message };
};

type Listeners = { [T in keyof ViewEvents]: ((event: ViewEvents[T]) => void)[] };

/** The components' names and levels, which decide each use's mount; their code doesn't. */
const shapeOf = (components: readonly HostComponent[]): string =>
  components.map(({ name, trust, module }) => `${name}:${trust.level}:${module ? "module" : "source"}`).join("\n");

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
      const listeners: Listeners = { set: [], link: [], error: [], rendered: [], loaded: [] };
      const emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => {
        for (const listener of listeners[type]) listener(event);
      };
      const observations = new Set<{ log: Log; listener: (events: readonly ObservedEvent[]) => void }>();
      const observer = new Observer((events, by) => {
        const marked = reportedBy(events, by);
        for (const { log, listener } of observations) {
          log.add(marked);
          listener(marked);
        }
      });
      const runtime = new HostRuntime(
        { ...options, observer },
        {
          set: (input, value) => {
            emit("set", { input, value });
          },
          link: (url) => {
            emit("link", { url });
          },
          error: (component, reason, message) => {
            emit("error", errorEvent(component, reason, message));
          },
        },
      );
      const page = options.page === "host" ? hostPage(container, runtime, observer, emit) : framePage(container, runtime, options, observer, emit);
      // The frames take the page's colour scheme: when the view opens, when the reader's preference changes, when
      // printing starts and ends, at each render, and when the host says it switched its own.
      let scheme: Scheme | undefined;
      const followScheme = () => {
        const next = schemeOf(container);
        if (next === scheme) return;
        scheme = next;
        runtime.scheme(next);
        page.scheme(next);
      };
      const preference = matchMedia("(prefers-color-scheme: dark)");
      preference.addEventListener("change", followScheme);
      addEventListener("beforeprint", followScheme);
      addEventListener("afterprint", followScheme);
      followScheme();
      /**
       * The page's components, compiled. Trees and splices wait for components of names or levels the page hasn't
       * had, so their uses mount with code; new code for the same ones swaps in where they are mounted.
       */
      // After `open` returns, so the listeners a host adds at once hear what the components' code reports.
      let ready = Promise.resolve()
        .then(() => runtime.set(components))
        .then(() => {
          page.code();
        });
      let shape = shapeOf(components);
      return {
        render: (tree, change) => {
          followScheme();
          void ready.then(() => {
            page.render(tree, change);
          });
        },
        splice: (splices, change, spliceOptions) => {
          void ready.then(() => {
            page.splice(splices, change, spliceOptions);
          });
        },
        components: (next) => {
          const loading = ready.then(async () => {
            await runtime.set(next);
            page.code();
          });
          const nextShape = shapeOf(next);
          if (nextShape !== shape) ready = loading;
          shape = nextShape;
          return loading;
        },
        theme: (tokens) => {
          runtime.theme(tokens);
          page.theme(tokens);
        },
        scheme: () => {
          followScheme();
          page.reread();
        },
        styles: (urls) => {
          runtime.styles(urls);
          page.styles(urls);
        },
        on: (type, listener) => {
          listeners[type].push(listener);
        },
        observe: (listener) => {
          const observation = { log: new Log(), listener };
          // A second observation starts with the host's page's live frames and components; the first, with frames' too.
          if (observer.on) observation.log.add(observer.live());
          observations.add(observation);
          observer.start();
          return {
            get events() {
              return observation.log.events;
            },
            get dropped() {
              return observation.log.dropped;
            },
            stop: () => {
              observations.delete(observation);
              if (observations.size === 0) observer.stop();
            },
          };
        },
        destroy: () => {
          observer.stop();
          observations.clear();
          preference.removeEventListener("change", followScheme);
          removeEventListener("beforeprint", followScheme);
          removeEventListener("afterprint", followScheme);
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
  styles(urls: readonly string[]): void;
  /** The page's colour scheme, for the page's own frame. */
  scheme(scheme: Scheme): void;
  /** The host switched its own theme: a page in the host's page reads its scheme again, for its trusted components. */
  reread(): void;
  destroy(): void;
}

type Emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => void;

/** Host mode: the page in the host's page, with the runtime's components. */
function hostPage(container: HTMLElement, runtime: HostRuntime, observer: Observer, emit: Emit): PageEnd {
  const page: Page = createPage(container, {
    components: runtime.mounts,
    onSet: (input, value) => {
      emit("set", { input, value });
    },
    onError: (error) => {
      if (!("component" in error)) return;
      const message = error.error instanceof Error ? error.error.message : "";
      observer.fallback(error.component, error.reason, message);
      emit("error", errorEvent(error.component, error.reason, message));
    },
  });
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
    destroy: () => {
      page.destroy();
    },
  };
}

/** Frame mode: the page in one strict frame, which renders it through `@compmd/render`'s frame protocol. */
function framePage(container: HTMLElement, runtime: HostRuntime, options: SandboxOptions, observer: Observer, emit: Emit): PageEnd {
  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts");
  iframe.setAttribute("allow", "");
  iframe.title = "The rendered page";
  iframe.className = "compmd-page-frame";
  iframe.style.cssText = "border: 0; width: 100%; height: 100%; display: block;";
  /** The document's inputs, which its page may set, from the last tree or splice. */
  let inputs: Readonly<Record<string, ExpandedInput>> = {};
  const { port1, port2 } = new MessageChannel();
  const id = observer.frame("strict", { kind: "page" });
  const port = observer.port(id, port1);
  const child: ChildEnd = connectChild(port, { level: "strict", binds: (input) => input in inputs, ping: { every: 1000, timeout: options.timeout ?? 5000 } });
  observer.connected(id, child);
  // `@compmd/render`'s protocol on the same port: its messages wait for the page's renderer, which starts after the level.
  const frame: FrameConnection = connectFrame(port);
  child.on("set", (event) => {
    emit("set", event);
  });
  child.on("link", (event) => {
    emit("link", event);
  });
  child.on("error", ({ reason, component, message }) => {
    emit("error", errorEvent(component ?? "", reason, message));
  });
  child.on("rendered", (event) => {
    emit("rendered", event);
  });
  child.on("loaded", (event) => {
    emit("loaded", event);
  });
  child.on("failed", (failure) => {
    observer.gone(id, failure.reason);
    emit("error", errorEvent("", "threw", failure.reason === "hung" ? "the page's frame stopped answering" : `the page's frame is of protocol version ${String(failure.version)}`));
  });
  if (options.stylesheets?.length) child.styles([...options.stylesheets]);
  iframe.addEventListener("load", () => iframe.contentWindow?.postMessage({ type: HELLO, version: SANDBOX_VERSION }, "*", [port2]), { once: true });
  iframe.src = frameUrl(options.base, "strict.html");
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
    styles: (urls) => {
      child.styles([...urls]);
    },
    scheme: (scheme) => {
      child.scheme(scheme);
    },
    // The page's frame takes its scheme from `scheme`.
    reread: () => undefined,
    destroy: () => {
      frame.close();
      child.close();
      iframe.remove();
      observer.gone(id, "removed");
    },
  };
}
