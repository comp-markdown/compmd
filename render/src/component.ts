// The component layer: one host-language component node in its outer box, run through the host contract
// (spec §7.4). Its caller renders the slots and the fallback, so a region and an editor's widget can both use it.
import type { ComponentError, Fallback, Mount, Mounted, Slots } from "./contract.ts";
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
  /** The component changed the input bound to one of its params. */
  onSet(input: string, value: unknown): void;
  /** The component shows its fallback instead: it is missing, or it threw (W118). */
  onError?(error: ComponentError): void;
  /** Shows the fallback in the box when the component can't run; by default `fallback.element` is placed there. */
  showFallback?(box: HTMLElement): void;
  host?: unknown;
  /** The document's stylesheet, which the outer box's params are written to; the caller flushes it. */
  stylesheet: Stylesheet;
  /** How the box's parent lays it out (default `block`). */
  parent?: ParentLayout;
}

/** What changes on an update besides the node: the slots the caller rendered again, and the parent's layout. */
export type ComponentUpdate = Pick<ComponentOptions, "slots"> & Partial<Pick<ComponentOptions, "parent">>;

export interface ComponentHandle {
  /** The outer box (spec §9.2), the component's `target`. */
  readonly element: HTMLElement;
  /** Whether the component shows its fallback. */
  readonly failed: boolean;
  update(node: ExpandedComponent, next: ComponentUpdate): void;
  destroy(): void;
}

/**
 * Mounts `node`'s component into a new outer box at the end of `target`. The box gets the common params (§9.2); the
 * component gets its own params, its slots, `set`, its fallback and its heading level, and owns what is inside the box.
 *
 * @example
 * const chart = mountComponent(widget, node, { mount: drawChart, slots: {}, fallback, headingLevel: 2, onSet, stylesheet });
 * stylesheet.flush();
 * chart.update(nextNode, { slots: {}, fallback });
 */
export function mountComponent(target: ParentNode & Node, node: ExpandedComponent, options: ComponentOptions): ComponentHandle {
  const document = target.ownerDocument ?? (target as Document);
  const element = document.createElement(node.type === "componentInline" ? "span" : "div");
  element.classList.add("compmd-component");
  target.appendChild(element);
  let current = node;
  let parent = options.parent ?? "block";
  let mounted: Mounted | undefined;
  let failed = false;
  let updating = false;
  let destroyed = false;

  const fail = (reason: ComponentError["reason"], error?: unknown) => {
    failed = true;
    mounted = undefined;
    element.replaceChildren();
    if (options.showFallback) options.showFallback(element);
    else element.appendChild(options.fallback.element);
    options.onError?.({ component: current.name, reason, ...(error === undefined ? {} : { error }) });
  };
  const set = (param: string, value: unknown) => {
    if (destroyed) return;
    if (updating) throw new Error(`${current.name}: set is refused during update`);
    const input = current.params.find((p) => p.key === param && p.breakpoint === null)?.input;
    if (input === undefined) throw new Error(`${current.name}: ${param} isn't bound to an input`);
    options.onSet(input, value);
  };

  applyBox(element, current.params, parent, options.stylesheet);
  if (!options.mount) fail("missing");
  else {
    try {
      mounted = options.mount(element, {
        params: ownParams(current.params),
        slots: options.slots,
        set,
        fallback: options.fallback,
        headingLevel: options.headingLevel,
        ...(options.host === undefined ? {} : { host: options.host }),
      }) ?? undefined;
    } catch (error) {
      fail("threw", error);
    }
  }

  return {
    element,
    get failed() {
      return failed;
    },
    update(next, { slots, parent: nextParent }) {
      current = next;
      parent = nextParent ?? parent;
      applyBox(element, next.params, parent, options.stylesheet);
      if (failed || !mounted?.update) return;
      updating = true;
      try {
        mounted.update(ownParams(next.params), slots);
      } catch (error) {
        fail("threw", error);
      } finally {
        updating = false;
      }
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
