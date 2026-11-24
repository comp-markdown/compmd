// `@compmd/pack/testing` (docs/writing-a-pack.md, decision 7): a component run through the host contract as the
// renderer runs it, in a DOM (happy-dom, as `compmd-pack test` gives), with no stubs for the frame's document.

import { mountComponent, Stylesheet, type ExpandedComponent, type Fit, type Mount, type Mounted, type Params, type Scheme, type Slots } from "@compmd/render";
import { messageOf } from "./message-of.ts";
import { SETTLED, type DrawMounted } from "./component.ts";

export interface TestOptions {
  params?: Params;
  /** The fallback's text, the component's text alternative; `Fallback` by default. */
  fallback?: string;
  /** The colour scheme it is shown in; `light` by default. */
  scheme?: Scheme;
  /** How its drawing sits in its box (`object-fit`, `object-position`), in a left-to-right document; contain, centred, by default. */
  fit?: Fit;
  /** The params bound to an input, which the component may `set`. */
  inputs?: readonly string[];
  slots?: Slots;
}

/** A component under test. */
export interface TestMount {
  /** Its box: what it drew, or the fallback. */
  readonly target: HTMLElement;
  update(params: Params): void;
  /** The scheme changed: the component gets it, and an update with the same params. */
  setScheme(scheme: Scheme): void;
  /** The box's fit changed: the component gets it, and an update with the same params. */
  setFit(fit: Fit): void;
  /**
   * Resolves once its draws are done (`@compmd/pack/component`'s) and what it gave `busy` has settled, or at once for
   * one that draws synchronously.
   */
  settled(): Promise<void>;
  /** What it `set`, in order. */
  readonly sets: { param: string; value: unknown }[];
  /** The messages it failed with (`fail`, or a throw in `update`), in order. */
  readonly failures: string[];
  destroy(): void;
}


const PLACES: Record<number, string> = { 0: "start", 0.5: "center", 1: "end" };

/** `fit` as the box params a document writes: `object-fit`, and `object-position`, the block position then the inline one. */
const fitParams = (fit: Fit | undefined): Params => (fit ? { "object-fit": fit.mode, "object-position": `${PLACES[fit.y] ?? "center"} ${PLACES[fit.x] ?? "center"}` } : {});

/** The leaf a test's params make, each bound to an input of its own name when `inputs` names it, in a box with `fit`. */
function leafOf(params: Params, inputs: readonly string[], fit?: Fit): ExpandedComponent {
  return {
    type: "componentLeaf",
    name: "component",
    params: Object.entries({ ...params, ...fitParams(fit) }).map(([key, value]) => ({ key, breakpoint: null, value, ...(inputs.includes(key) ? { input: key } : {}) })),
    fallback: [],
  };
}

/**
 * Mounts `component` in the document, as the renderer does: a failure shows the fallback until the next update.
 *
 * @example
 * const chart = mountForTest(mermaid, { params: { code: "graph LR; A --> B", theme: "auto" }, scheme: "dark" });
 * await chart.settled();
 * expect(chart.target.querySelector("svg")).not.toBeNull();
 */
export function mountForTest(component: Mount, options: TestOptions = {}): TestMount {
  const inputs = options.inputs ?? [];
  const slots = options.slots ?? {};
  const stylesheet = new Stylesheet(document);
  const holder = document.body.appendChild(document.createElement("div"));
  const sets: TestMount["sets"] = [];
  const failures: string[] = [];
  /** What the component said it was drawing (`busy`). */
  const drawing: Promise<unknown>[] = [];
  let mounted: Mounted | undefined;
  const watched: Mount = (target, context) => {
    mounted = component(target, context) ?? undefined;
    return mounted;
  };
  const fallback = Object.assign(document.createElement("span"), { textContent: options.fallback ?? "Fallback" });
  let params = options.params ?? {};
  let fit = options.fit;
  const handle = mountComponent(holder, leafOf(params, inputs, fit), {
    mount: watched,
    slots,
    fallback: { text: fallback.textContent, element: fallback },
    headingLevel: 2,
    scheme: options.scheme ?? "light",
    onSet: (param, value) => sets.push({ param, value }),
    onError: ({ error }) => failures.push(messageOf(error)),
    onBusy: (work) => drawing.push(work.catch(() => undefined)),
    stylesheet,
  });
  return {
    target: handle.element,
    update(next) {
      params = next;
      handle.update(leafOf(params, inputs, fit), { slots });
    },
    setScheme(scheme) {
      handle.scheme(scheme);
    },
    setFit(next) {
      fit = next;
      handle.update(leafOf(params, inputs, fit), { slots });
    },
    async settled() {
      if (mounted && SETTLED in mounted) await (mounted as DrawMounted)[SETTLED]();
      await Promise.all(drawing);
    },
    sets,
    failures,
    destroy() {
      handle.destroy();
      holder.remove();
      stylesheet.remove();
    },
  };
}
