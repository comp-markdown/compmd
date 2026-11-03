// The component layer: one host-language component node in its outer box, run through the host contract
// (spec §7.4). Its caller renders the slots and the fallback, so a region and an editor's widget can both use it.
import type { ComponentError, Fallback, Mount, Mounted, MountContext, Scheme, Slots } from "./contract.ts";
import { applyBox } from "./box.ts";
import { ownParams, type ParentLayout, type Stylesheet } from "./styles.ts";
import type { ExpandedComponent } from "./tree.ts";

export interface ComponentOptions {
  /** The component; without one it shows its fallback, as a component not available here. */
  mount: Mount | undefined;
  /** Each slot's content, rendered by the caller. */
  slots: Slots;
  fallback: Fallback;
  /** The level the component's top heading should have (spec §7.2). */
  headingLevel: number;
  /** The colour scheme the component is shown in (default `light`); `ComponentHandle.scheme` changes it. */
  scheme?: Scheme;
  /** The component changed the input bound to one of its params. */
  onSet(input: string, value: unknown): void;
  /** The component shows its fallback instead: it is missing, it threw, or it failed (W118). */
  onError?(error: ComponentError): void;
  /**
   * Shows the fallback in the box when the component can't run; by default `fallback.element` is placed there. It
   * `lasts` when the component never mounted; otherwise the next update takes the box back, after `hideFallback`.
   */
  showFallback?(box: HTMLElement, lasts: boolean): void;
  /** The box is the component's again, after a failure that didn't last: what `showFallback` made can go. */
  hideFallback?(box: HTMLElement): void;
  host?: unknown;
  /** The document's stylesheet, which the outer box's params are written to; the caller flushes it. */
  stylesheet: Stylesheet;
  /** How the box's parent lays it out (default `block`). */
  parent?: ParentLayout;
  /**
   * Runs the mount later, once the caller has placed the box in the document and flushed the stylesheet, so the
   * component mounts into a box with its styles, its theme and its place; by default it mounts at once.
   */
  later?: (mount: () => void) => void;
}

/** What changes on an update besides the node: the slots the caller rendered again, and the parent's layout. */
export type ComponentUpdate = Pick<ComponentOptions, "slots"> & Partial<Pick<ComponentOptions, "parent">>;

export interface ComponentHandle {
  /** The outer box (spec §9.2), the component's `target`. */
  readonly element: HTMLElement;
  /** Whether the component never mounted, so it shows its fallback for good. */
  readonly failed: boolean;
  update(node: ExpandedComponent, next: ComponentUpdate): void;
  /** The colour scheme changed: the component gets it, and an update with the same params and slots. */
  scheme(scheme: Scheme): void;
  destroy(): void;
}

/**
 * Mounts `node`'s component into a new outer box at the end of `target`. The box gets the common params (§9.2); the
 * component gets its own params, its slots, `set`, `fail`, its fallback, its heading level and its colour scheme, and
 * owns what is inside the box. A component that fails, or throws while updating, shows its fallback until its next
 * update; one that throws while mounting shows it for good.
 *
 * @example
 * const chart = mountComponent(widget, node, { mount: drawChart, slots: {}, fallback, headingLevel: 2, onSet, stylesheet });
 * stylesheet.flush();
 * chart.update(nextNode, { slots: {}, fallback });
 * chart.scheme("dark");
 */
export function mountComponent(target: ParentNode & Node, node: ExpandedComponent, options: ComponentOptions): ComponentHandle {
  const document = target.ownerDocument ?? (target as Document);
  const element = document.createElement(node.type === "componentInline" ? "span" : "div");
  element.classList.add("compmd-component");
  target.appendChild(element);
  let current = node;
  let slots = options.slots;
  let parent = options.parent ?? "block";
  let mounted: Mounted | undefined;
  /** The component never mounted. */
  let failed = false;
  /** The fallback is in the box until the next update. */
  let showing = false;
  let updating = false;
  let destroyed = false;

  const showFallback = (lasts: boolean) => {
    element.replaceChildren();
    if (options.showFallback) options.showFallback(element, lasts);
    else element.appendChild(options.fallback.element);
  };
  const giveUp = (reason: ComponentError["reason"], error?: unknown) => {
    failed = true;
    mounted = undefined;
    showFallback(true);
    options.onError?.({ component: current.name, reason, ...(error === undefined ? {} : { error }) });
  };
  const fallBack = (error: unknown) => {
    if (destroyed || failed || showing) return;
    showing = true;
    showFallback(false);
    options.onError?.({ component: current.name, reason: "threw", error });
  };
  const set = (param: string, value: unknown) => {
    if (destroyed) return;
    if (updating) throw new Error(`${current.name}: set is refused during update`);
    const input = current.params.find((p) => p.key === param && p.breakpoint === null)?.input;
    if (input === undefined) throw new Error(`${current.name}: ${param} isn't bound to an input`);
    options.onSet(input, value);
  };
  const context: MountContext = {
    params: ownParams(current.params),
    slots,
    set,
    fallback: options.fallback,
    headingLevel: options.headingLevel,
    scheme: options.scheme ?? "light",
    fail: (message) => {
      fallBack(new Error(message));
    },
    ...(options.host === undefined ? {} : { host: options.host }),
  };
  const runUpdate = () => {
    if (failed || !mounted?.update) return;
    if (showing) {
      showing = false;
      options.hideFallback?.(element);
      element.replaceChildren();
    }
    updating = true;
    try {
      mounted.update(ownParams(current.params), slots);
    } catch (error) {
      fallBack(error);
    } finally {
      updating = false;
    }
  };

  applyBox(element, current.params, parent, options.stylesheet);
  const mount = options.mount;
  if (!mount) giveUp("missing");
  else {
    const run = () => {
      if (destroyed) return;
      // An update before a later mount: the component mounts with what it gives.
      context.params = ownParams(current.params);
      context.slots = slots;
      try {
        mounted = mount(element, context) ?? undefined;
      } catch (error) {
        giveUp("threw", error);
      }
    };
    if (options.later) options.later(run);
    else run();
  }

  return {
    element,
    get failed() {
      return failed;
    },
    update(next, { slots: nextSlots, parent: nextParent }) {
      current = next;
      slots = nextSlots;
      parent = nextParent ?? parent;
      applyBox(element, next.params, parent, options.stylesheet);
      runUpdate();
    },
    scheme(next) {
      if (next === context.scheme) return;
      context.scheme = next;
      runUpdate();
    },
    destroy() {
      destroyed = true;
      try {
        mounted?.destroy?.();
      } catch (error) {
        options.onError?.({ component: current.name, reason: "threw", error });
      }
      options.stylesheet.forget(element);
      element.remove();
    },
  };
}
