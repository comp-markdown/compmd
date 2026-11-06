// The sandbox's default for a whole document (docs/running-components-in-the-sandbox.md, Q1, Q7): the host chooses
// where the page renders. In frame mode the page is one strict frame per document; in host mode the page renders in
// the host's page, trusted components inline, every other use in a frame of its level. Either way the host gives
// trees and gets `set`, links and errors, and never writes a frame or a policy itself.

import type { HostComponent } from "@compmd/processor";
import { schemeOf, type Change, type ExpandedInput, type ExpandedRoot, type Line, type Source, type Splice } from "@compmd/render";
import type { SpliceOptions } from "@compmd/render/frame";
import { HostRuntime, type HostRuntimeOptions } from "./host/runtime.ts";
import { Log, reportedBy, type Observation, type ObservedEvent } from "./observation.ts";
import { Observer } from "./observer.ts";
import { errorEvent, framePage, hostPage } from "./pages.ts";
import type { ErrorReason, SandboxDiagnostic, Scheme } from "./protocol/messages.ts";
import { Settling } from "./settling.ts";

export interface SandboxOptions extends HostRuntimeOptions {
  /** Where the page renders: in a strict frame per document (the default), or in the host's page. */
  page?: "frame" | "host";
}

export interface ViewEvents {
  set: { input: string; value: unknown };
  /** A reader clicked a link in the page; the host shows its URL before it opens (spec §15 rule 4). */
  link: { url: string };
  /**
   * A component shows its fallback, why, and the diagnostic it is when it is one (`diagnosticOf`). `use` names the
   * use, an opaque id, when it has one: a later `recovered` with the same `use` says it draws again.
   */
  error: { component: string; reason: ErrorReason; code?: SandboxDiagnostic; message: string; use?: string };
  /** A use that failed draws again: what the host listed for its `error`, with the same `use`, can go. */
  recovered: { component: string; use: string };
  /** A render or splice is painted, `ms` after it started. */
  rendered: { ms: number };
  /** The components last given are loaded in the page, or show their fallbacks: a host may show the page from now. */
  loaded: Record<string, never>;
  /**
   * The reader clicked in the page, on what came from `source` (as `@compmd/render`'s `Page.sourceOf` gives it): an
   * editor moves its caret there. Only once the host listens: the first listener turns it on, so a page doesn't
   * report its reader's clicks unasked. The click itself goes on as usual, a link to `link`.
   */
  pointed: { source: Source };
  /** What the page shows may have moved, so its `lines()` may have: as `@compmd/render`'s `onLayout`. */
  layout: Record<string, never>;
  /** The page has settled again (`View.settled`), after a render, a splice, or something that moved it. */
  settled: Record<string, never>;
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
  /**
   * Sends the host's event `event` with `data`, which must be copyable, to every component given `host` (spec §7.4):
   * the trusted ones in the host's page, and those of permissive packs given it, which hear it with `host.on`. Strict
   * components never do.
   *
   * @example
   * view.emit("ratings-changed", { product: "p-1" });
   */
  emit(event: string, data: unknown): void;
  /**
   * Scrolls the page, as little as it takes, to what the source at `offset` drew (`Page.elementsAt`), and marks it
   * briefly (`reveal`, class `compmd-revealed`). Nothing when nothing the page shows came from there. In order with
   * renders and splices: after the last one sent.
   *
   * @param offset - A code-point offset in the text of the last render or splice; an editor's caret through
   * `codePointOffset`.
   *
   * @example
   * editor.onCaret((caret) => view.reveal(codePointOffset(text, caret)));
   */
  reveal(offset: number): void;
  /**
   * The boxes of what the source at `offset` drew, one per element, in the host's viewport (as
   * `getBoundingClientRect()` gives them), for a host that draws its own marks over the page. In frame mode the
   * frame measures them and the frame's place is added; they are as of when the frame answered. Empty when nothing
   * came from there, or the view is gone. In order with renders and splices, as `reveal`.
   *
   * @example
   * const [box] = await view.rectsAt(offset);
   * if (box) Object.assign(marker.style, { top: `${box.top + scrollY}px`, height: `${box.height}px` });
   */
  rectsAt(offset: number): Promise<DOMRect[]>;
  /**
   * Each line of the text of the last render or splice that draws something, from line `from` (1 without it), with the
   * boxes it shows in, one per visual line it covers, in CSS pixels from the top left corner of the container's
   * content, by `@compmd/render`'s rules (its `Page.lines`): for a gutter of line numbers. Each line is measured as
   * the loop reaches it; in frame mode the frame measures a batch at a time, of its page as if it weren't scrolled,
   * and where the frame is in the container is added. A use in a frame of its own is placed as a whole, on its first
   * line. `layout` says when to ask again.
   *
   * @example
   * view.on("layout", async () => {
   *   for await (const { line, rects } of view.lines()) place(line, rects);
   * });
   */
  lines(from?: number): AsyncGenerator<Line>;
  /**
   * Resolves the next time the page has settled, at once when it has: after the latest render or splice, every use in
   * the host's page and in every frame at any depth mounted or showing its fallback, every drawing a component gave
   * `busy` settled (spec §7.4), every frame settled with its last size applied, the page's images loaded or failed and
   * its fonts loaded, then an animation frame in which nothing moved. The container has `data-compmd-settled` while it
   * is settled, and `settled` comes each time it settles again. A frame that hangs is stopped and counts as settled; a
   * drawing or a frame that never says it is done holds it up for `BUSY_MS` at most.
   *
   * @example
   * view.render(processed.expanded);
   * await view.settled();   // safe to click, print, or scroll to a link's target
   */
  settled(): Promise<void>;
  /**
   * Listens for `type`; a listener added as soon as `open` returns hears everything, opening's errors too. The first
   * listener for `pointed` asks the page to report clicks.
   */
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
      const listeners: Listeners = { set: [], link: [], error: [], recovered: [], rendered: [], loaded: [], pointed: [], layout: [], settled: [] };
      /** What the page shows changed or moved: it isn't settled until it settles again. */
      let changed: () => void = () => undefined;
      const emit = <T extends keyof ViewEvents>(type: T, event: ViewEvents[T]) => {
        if (type === "layout") changed();
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
          error: (component, reason, message, use) => {
            emit("error", errorEvent(component, reason, message, use));
          },
          recovered: (component, use) => {
            emit("recovered", { component, use });
          },
        },
      );
      const page = options.page === "host" ? hostPage(container, runtime, options, observer, emit) : framePage(container, runtime, options, observer, emit);
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
      const settling = new Settling(
        {
          drawn: () => ready.then(() => page.drawn()),
          frames: () => page.frames(),
          root: () => container,
        },
        () => {
          container.dataset.compmdSettled = "";
          emit("settled", {});
        },
      );
      changed = () => {
        delete container.dataset.compmdSettled;
        settling.changed();
      };
      return {
        render: (tree, change) => {
          followScheme();
          changed();
          void ready.then(() => {
            page.render(tree, change);
          });
        },
        splice: (splices, change, spliceOptions) => {
          changed();
          void ready.then(() => {
            page.splice(splices, change, spliceOptions);
          });
        },
        components: (next) => {
          changed();
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
          changed();
          runtime.theme(tokens);
          page.theme(tokens);
        },
        scheme: () => {
          changed();
          followScheme();
          page.reread();
        },
        styles: (urls) => {
          changed();
          runtime.styles(urls);
          page.styles(urls);
        },
        emit: (event, data) => {
          runtime.emit(event, data);
        },
        reveal: (offset) => {
          void ready.then(() => {
            page.reveal(offset);
          });
        },
        rectsAt: (offset) => ready.then(() => page.rectsAt(offset)),
        lines: async function* (from) {
          await ready;
          yield* page.lines(from);
        },
        settled: () => settling.whenSettled(),
        on: (type, listener) => {
          if (type === "pointed" && listeners.pointed.length === 0) page.point();
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

