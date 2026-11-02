// `@compmd/pack/component` (docs/writing-a-pack.md, decision 6): what a component that draws asynchronously would
// otherwise write itself, bundled into its module. A component that draws synchronously writes plain `mount`.

import type { Mount, MountContext, Mounted, Params, Scheme, Slots } from "@compmd/render";

/** The host contract's types (spec §7.4). */
export type { Fallback, Mount, MountContext, Mounted, Params, Scheme, Slots } from "@compmd/render";

/** The contract's context, its params typed as the component's signature declares them. */
export interface Context<P> extends Omit<MountContext, "params"> {
  params: P;
}

/** What a component's `mount` returns, its params typed. */
export interface Updates<P> {
  update?(params: P, slots: Slots): void;
  destroy?(): void;
}

/**
 * A component written as plain `mount` (spec §7.6), its params typed: `P` is what its signature declares, which the
 * processor has checked before the component sees them.
 *
 * @example
 * export default component<{ value: string }>((target, { params }) => {
 *   target.textContent = params.value;
 *   return { update: (next) => { target.textContent = next.value; } };
 * });
 */
// A mount that needs no update or destroy returns nothing, as the plain binding allows.
// eslint-disable-next-line @typescript-eslint/no-invalid-void-type
export function component<P = Params>(mount: (target: Element, context: Context<P>) => Updates<P> | undefined | void): Mount {
  return mount as unknown as Mount;
}

/** What a drawing is made with besides its params. */
export interface DrawContext {
  /** The colour scheme the component is shown in (spec §7.4). */
  scheme: Scheme;
  /** Aborted when a newer draw overtakes this one, or the component goes. */
  signal: AbortSignal;
}

/** What a draw gives: an element, or SVG or HTML as text. */
export type Drawing = Element | string;

export interface DrawOptions {
  /** The drawing's role, `img` for a diagram: the drawing gets it, with the fallback's text as its label. */
  role?: string;
  /**
   * Draws one at a time, for a library with one global configuration, as Mermaid has: each draw waits for the one
   * before, so two components' themes never mix. One queue per frame, which loads the module once.
   */
  oneAtATime?: boolean;
}

/** Where `@compmd/pack/testing` finds a component's draws, to wait for them. */
export const SETTLED = Symbol.for("compmd.pack.settled");

/** A mounted component `draw` made: its draws can be waited for. */
export interface DrawMounted extends Mounted {
  [SETTLED](): Promise<void>;
}

/** The draws waiting to run, for `oneAtATime`. */
let queue: Promise<unknown> = Promise.resolve();

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/**
 * A component that draws with `render`, which may take its time: the newest params win, a draw that a newer one
 * overtakes is dropped, the drawing replaces the last one only once it is ready, and a draw that throws calls
 * `context.fail` with its message (spec §7.4), so the fallback shows until the next update. `render` runs again
 * when the scheme changes.
 *
 * @example
 * export default draw<{ code: string }>(async ({ code }, { scheme }) => (await library.render(code, { dark: scheme === "dark" })).svg, { role: "img" });
 */
// `P` types the params `render` destructures, as the component's signature declares them.
// eslint-disable-next-line @typescript-eslint/no-unnecessary-type-parameters
export function draw<P = Params>(render: (params: P, context: DrawContext) => Drawing | Promise<Drawing>, options: DrawOptions = {}): Mount {
  return (target, context): DrawMounted => {
    const drawing = target.ownerDocument.createElement("div");
    if (options.role !== undefined) drawing.setAttribute("role", options.role);
    let current: AbortController | undefined;
    let settled: Promise<void> = Promise.resolve();
    const run = (params: Params) => {
      current?.abort();
      const own = new AbortController();
      current = own;
      const job = async () => {
        // A draw waiting its turn may be overtaken before it starts.
        own.signal.throwIfAborted();
        const drawn = await render(params as P, { scheme: context.scheme, signal: own.signal });
        if (own.signal.aborted) return;
        if (options.role !== undefined) drawing.setAttribute("aria-label", context.fallback.text);
        if (typeof drawn === "string") drawing.innerHTML = drawn;
        else drawing.replaceChildren(drawn);
        // After a failure, the box was the renderer's: the drawing goes back into it.
        if (drawing.parentNode !== target) target.replaceChildren(drawing);
      };
      const attempt = options.oneAtATime ? queue.then(job) : job();
      if (options.oneAtATime) queue = attempt.catch(() => undefined);
      settled = attempt.catch((error: unknown) => {
        if (!own.signal.aborted) context.fail(messageOf(error));
      });
    };
    run(context.params);
    return {
      update: (params) => {
        run(params);
      },
      destroy: () => {
        current?.abort();
        target.replaceChildren();
      },
      [SETTLED]: () => settled,
    };
  };
}
