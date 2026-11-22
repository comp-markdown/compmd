// Inline styles the host's page refuses (docs/component-environment.md, decision 6): the host contract promises a
// component that inline styles apply in its box (spec §7.4), which a host's page whose policy refuses them breaks for
// the trusted components it mounts. While one is mounted, each refusal of an inline style is reported, naming the
// component whose box holds what was refused. A library that writes outside the box, such as one that measures in a
// temporary element in the page's body, gets a refusal of no box, which names no component.

import type { Mount, Mounted } from "@compmd/render";

/** An inline style the host's page refused while trusted components were mounted there. */
export interface RefusedStyle {
  /** The component whose box holds what was refused; absent when no box does. */
  component?: string;
  /** The directive that refused it: `style-src`, `style-src-attr` or `style-src-elem`. */
  directive: string;
  /** The start of what was refused, as the browser gives it; empty when it gives none. */
  sample: string;
}

const STYLE_DIRECTIVES = new Set(["style-src", "style-src-attr", "style-src-elem"]);

/** The element `node` is in, across a shadow root's edge. */
function parentOf(node: Node): Element | null {
  if (node.parentElement) return node.parentElement;
  const root = node.getRootNode();
  return root instanceof ShadowRoot ? root.host : null;
}

/**
 * The trusted components' boxes in the host's page, and the refusals of inline styles while any is mounted.
 *
 * @example
 * const refused = new RefusedStyles((style) => { console.warn(style); });
 * const components = { stars: refused.watch("stars", stars) };
 */
export class RefusedStyles {
  /** Each mounted box, with its component's name. */
  readonly #boxes = new Map<Element, string>();
  readonly #report: (refused: RefusedStyle) => void;
  readonly #listener = (event: SecurityPolicyViolationEvent) => {
    this.#refused(event);
  };
  /** The document listened to, while a box is mounted. */
  #listening: Document | undefined;

  constructor(report: (refused: RefusedStyle) => void) {
    this.#report = report;
  }

  /**
   * `mount` with its box watched while it is mounted. A mount that carries more than a function (`rendersSlots` marks
   * one) is left as it is: the renderer reads what it carries.
   */
  watch(name: string, mount: Mount): Mount {
    if (Object.getOwnPropertySymbols(mount).length > 0) return mount;
    return (target, context) => {
      this.#add(target, name);
      let mounted: Mounted | undefined;
      try {
        mounted = mount(target, context) ?? undefined;
      } catch (error) {
        this.#remove(target);
        throw error;
      }
      return {
        ...(mounted?.update ? { update: mounted.update.bind(mounted) } : {}),
        destroy: () => {
          this.#remove(target);
          mounted?.destroy?.();
        },
      };
    };
  }

  #add(box: Element, name: string): void {
    this.#boxes.set(box, name);
    if (this.#listening) return;
    this.#listening = box.ownerDocument;
    this.#listening.addEventListener("securitypolicyviolation", this.#listener);
  }

  #remove(box: Element): void {
    this.#boxes.delete(box);
    if (this.#boxes.size > 0) return;
    this.#listening?.removeEventListener("securitypolicyviolation", this.#listener);
    this.#listening = undefined;
  }

  #refused(event: SecurityPolicyViolationEvent): void {
    if (event.disposition !== "enforce" || !STYLE_DIRECTIVES.has(event.effectiveDirective)) return;
    const component = event.target instanceof Node ? this.#boxOf(event.target) : undefined;
    this.#report({ ...(component === undefined ? {} : { component }), directive: event.effectiveDirective, sample: event.sample });
  }

  /** The component whose box holds `node`, the innermost when boxes nest. */
  #boxOf(node: Node): string | undefined {
    for (let at: Node | null = node; at; at = parentOf(at)) {
      const name = at instanceof Element ? this.#boxes.get(at) : undefined;
      if (name !== undefined) return name;
    }
    return undefined;
  }
}
