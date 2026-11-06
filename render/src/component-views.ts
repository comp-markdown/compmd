// The views of components (spec §7.4, §7.7, §8.7, §9): `box` with its layout, the input components, host-language
// components with their slots, and code blocks a code renderer draws.
import { applyBox } from "./box.ts";
import { mountComponent, type ComponentHandle, type ComponentOptions } from "./component.ts";
import type { Fallback, Mount, SlotInstance, SlotItem, Slots } from "./contract.ts";
import { plainText } from "./document.ts";
import { withHtml } from "./html.ts";
import { CodeBlockView } from "./markdown-views.ts";
import { nodesMountOf, runnable } from "./nodes.ts";
import { createView, destroyAll, Matcher, patch, shapeOf, type Pass, type Surface, type View } from "./render.ts";
import { fills, layoutOf, ownParams, usesBreakpoints, type ParentLayout } from "./styles.ts";
import { contentOf, isComponent, rendererOf, textOf, type ExpandedCode, type ExpandedComponent, type ExpandedInstance, type ExpandedNode, type MarkdownNode } from "./tree.ts";

/** The input components, by name, with the kind of control each is (spec §8.7). */
export const INPUTS: Record<string, string> = { "date-input": "date", "number-input": "number", "text-input": "text", "select-input": "select", "toggle-input": "checkbox" };

/**
 * A code block a code renderer draws (spec §7.7): the renderer's component in its box, the block's text among its
 * params as `code` and `alt` as its fallback text; the code block is its fallback.
 */
export class RenderedCodeView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly #surface: Surface;
  readonly #handle: ComponentHandle;
  readonly #code: View;
  /** The use a mount that renders it itself gets: the code block, whose `alt` and code a sandbox's frame needs too. */
  #current: { node: ExpandedCode; inputs: Pass["inputs"] };

  constructor(surface: Surface, node: MarkdownNode, renderer: ExpandedComponent, pass: Pass, key: string | undefined) {
    this.shape = shapeOf(node);
    this.key = key;
    this.#current = { node: node as ExpandedCode, inputs: pass.inputs };
    this.#code = new CodeBlockView(surface, plainCode(node), undefined);
    const fallback = surface.document.createElement("div");
    fallback.appendChild(this.#code.dom);
    this.#surface = surface;
    const mount = surface.components[renderer.name];
    // The component layer appends the box to its target; the region places it, so it starts in a fragment.
    this.#handle = mountComponent(surface.document.createDocumentFragment(), renderer, {
      ...fromSurface(surface),
      mount: runnable(mount, () => this.#current),
      slots: {},
      fallback: { text: textOf((node as ExpandedCode).alt), element: fallback },
      headingLevel: Math.min(6, pass.heading.level + 1),
      parent: pass.parent,
      later: surface.pending.later,
    });
    followScheme(surface, this.#handle, mount);
  }

  get dom(): Node {
    return this.#handle.element;
  }

  update(node: ExpandedNode, pass: Pass): void {
    this.#code.update(plainCode(node), pass);
    const renderer = rendererOf(node);
    if (!renderer) return;
    this.#current = { node: node as ExpandedCode, inputs: pass.inputs };
    this.#handle.update(renderer, { slots: {}, parent: pass.parent });
  }

  destroy(): void {
    this.#surface.schemes.unfollow(this.#handle);
    this.#handle.destroy();
    this.#code.destroy();
  }
}

/** What every component of the surface is mounted with: its scheme, the caller's callbacks, `host` and the stylesheet. */
function fromSurface(surface: Surface): Pick<ComponentOptions, "scheme" | "onSet" | "onError" | "onRecovered" | "onBusy" | "host" | "stylesheet"> {
  return {
    scheme: surface.schemes.current,
    onSet: surface.onSet,
    ...(surface.onError ? { onError: surface.onError } : {}),
    ...(surface.onRecovered ? { onRecovered: surface.onRecovered } : {}),
    onBusy: (work) => {
      surface.busy.add(work);
    },
    host: surface.host,
    stylesheet: surface.stylesheet,
  };
}

/**
 * Lets `handle` follow the page's colour scheme. A mount that renders its slots is a frame, whose own renderer
 * follows the scheme the frame's document takes from the page.
 */
function followScheme(surface: Surface, handle: ComponentHandle, mount: Mount | undefined): void {
  if (!nodesMountOf(mount)) surface.schemes.follow(handle);
}

/** A code block as content, as a plain renderer shows it: without its code renderer. */
function plainCode(node: ExpandedNode): MarkdownNode {
  const { lang, meta, value } = node as ExpandedCode;
  return { type: "code", lang, meta, value };
}

/** The blocks placed in a box: down through Markdown containers, stopping at the next component (its own box). */
export function placedIn(nodes: readonly ExpandedNode[]): ExpandedComponent[] {
  return nodes.flatMap((node) => {
    if (isComponent(node)) return [node];
    const renderer = rendererOf(node);
    return renderer ? [renderer] : placedIn(node.children ?? []);
  });
}

/** Whether blocks placed in the box around `nodes` use breakpoints, so that box must be a container. */
export function needsContainer(nodes: readonly ExpandedNode[]): boolean {
  return placedIn(nodes).some((child) => usesBreakpoints(child.params));
}

/** Marks `box` as the container its blocks' breakpoints query, and as a column when one of them fills it. */
export function markBox(box: HTMLElement, nodes: readonly ExpandedNode[], layout: ParentLayout): void {
  box.classList.toggle("compmd-container", needsContainer(nodes));
  box.classList.toggle("compmd-fills", layout === "block" && placedIn(nodes).some((child) => fills(child.params)));
}

/** `box` (spec §9): a box with its layout, holding its children. */
export class BoxView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  #children: View[] = [];

  constructor(surface: Surface, node: ExpandedComponent, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement(node.type === "componentInline" ? "span" : "div");
    this.dom.classList.add("compmd-box");
    this.update(node, pass);
  }

  update(node: ExpandedNode, pass: Pass): void {
    const box = node as ExpandedComponent;
    const layout = layoutOf(box.params);
    applyBox(this.dom, box.params, pass.parent, this.#surface.stylesheet, true);
    const children = box.children ?? [];
    markBox(this.dom, children, layout);
    this.#children = patch(this.#surface, this.dom, this.#children, children, { ...pass, parent: layout });
  }

  destroy(): void {
    destroyAll(this.#children);
    this.#surface.stylesheet.forget(this.dom);
  }
}

/** An input's visible label: its fallback before the first colon, since a fallback writes the default after one ("From: today"). */
export function inputLabel(fallback: string): string {
  const colon = fallback.indexOf(":");
  return (colon === -1 ? fallback : fallback.slice(0, colon)).trim();
}

/**
 * An input component (spec §8.7): a form control showing its input's value, in a label from its fallback; changing it
 * sets the input.
 */
export class InputView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  readonly #label: HTMLElement;
  readonly #control: HTMLInputElement | HTMLSelectElement;
  readonly #kind: string;
  #input = "";

  constructor(surface: Surface, node: ExpandedComponent, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement("label");
    this.dom.classList.add("compmd-box", "compmd-input");
    this.#label = this.dom.appendChild(surface.document.createElement("span"));
    this.#label.className = "compmd-input-label";
    this.#kind = INPUTS[node.name] ?? "text";
    if (this.#kind === "select") this.#control = surface.document.createElement("select");
    else {
      const input = surface.document.createElement("input");
      input.type = this.#kind;
      this.#control = input;
    }
    this.dom.appendChild(this.#control);
    const live = this.#kind === "text" || this.#kind === "number";
    this.#control.addEventListener(live ? "input" : "change", () => {
      surface.onSet(this.#input, this.#read());
    });
    this.update(node, pass);
  }

  #read(): unknown {
    const control = this.#control;
    if (this.#kind === "checkbox") return (control as HTMLInputElement).checked;
    if (this.#kind === "number") return Number(control.value);
    return control.value;
  }

  update(node: ExpandedNode, pass: Pass): void {
    const input = node as ExpandedComponent;
    applyBox(this.dom, input.params, pass.parent, this.#surface.stylesheet);
    const bind = input.params.find((p) => p.key === "bind");
    this.#input = bind?.input ?? textOf(bind?.value);
    const declared = pass.inputs[this.#input];
    const label = inputLabel(plainText(input.fallback ?? []));
    if (this.#label.textContent !== label) this.#label.textContent = label;
    const control = this.#control;
    if (this.#kind === "select") {
      const members = declared?.members ?? [];
      const options = [...(control as HTMLSelectElement).options].map((option) => option.value);
      if (options.join("\u0000") !== members.join("\u0000")) {
        control.replaceChildren(...members.map((member) => Object.assign(this.#surface.document.createElement("option"), { value: member, textContent: member })));
      }
    }
    const value = declared?.value;
    if (this.#kind === "checkbox") (control as HTMLInputElement).checked = value === true;
    // While the reader types, the control keeps what they typed.
    else if (control !== this.#surface.document.activeElement && control.value !== textOf(value)) control.value = textOf(value);
  }

  destroy(): void {
    this.#surface.stylesheet.forget(this.dom);
  }
}

// ── host-language components and their slots ───────────────────────────────────

/** One block of a slot, in the box the renderer makes for it (spec §7.4, §9.1). */
interface ItemView {
  element: HTMLElement;
  view: View;
  node: ExpandedNode;
}

type SlotView = { items: ItemView[] } | { instances: { value: unknown; items: ItemView[] }[] };

function itemsOf(slot: SlotView): ItemView[] {
  return "items" in slot ? slot.items : slot.instances.flatMap((instance) => instance.items);
}

function slotItem({ element, node }: ItemView): SlotItem {
  if (node.type === "componentPending") return { element, component: { name: textOf(node.name), params: {}, pending: true } };
  return isComponent(node) ? { element, component: { name: node.name, params: ownParams(node.params) } } : { element };
}

export class ComponentView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly #surface: Surface;
  readonly #handle: ComponentHandle;
  #slots = new Map<string, SlotView>();
  /** The fallback's views: a leaf's fallback, or a failed container's content. */
  #fallback: View[] = [];
  readonly #fallbackElement: HTMLElement;
  #node: ExpandedComponent;
  #inputs: Pass["inputs"];
  /** Whether the component renders its slots itself, from their nodes (`rendersSlots`). */
  readonly #ownSlots: boolean;

  constructor(surface: Surface, node: ExpandedComponent, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.#node = node;
    this.#inputs = pass.inputs;
    const mount = surface.components[node.name];
    this.#ownSlots = nodesMountOf(mount) !== undefined;
    const headingLevel = Math.min(6, pass.heading.level + 1);
    this.#fallbackElement = surface.document.createElement(node.fallback ? "span" : "div");
    // Its mount comes before those of the components in its slots, which it places as it mounts.
    const later = surface.pending.reserve();
    const fallback = this.#renderFallback(node, pass);
    const slots = this.#ownSlots ? {} : this.#patchSlots(node, pass);
    // The component layer appends the box to its target; the region places it, so it starts in a fragment.
    this.#handle = mountComponent(surface.document.createDocumentFragment(), node, {
      ...fromSurface(surface),
      mount: runnable(mount, () => ({ node: this.#node, inputs: this.#inputs })),
      slots,
      fallback,
      headingLevel,
      showFallback: (box, lasts) => {
        this.#showFallback(box, lasts, pass);
      },
      hideFallback: () => {
        this.#hideFallback();
      },
      parent: pass.parent,
      later,
    });
    followScheme(surface, this.#handle, mount);
  }

  get dom(): Node {
    return this.#handle.element;
  }

  update(node: ExpandedNode, pass: Pass): void {
    const next = node as ExpandedComponent;
    this.#node = next;
    this.#inputs = pass.inputs;
    if (this.#handle.failed) {
      this.#handle.update(next, { slots: {}, parent: pass.parent });
      const into = next.fallback ? this.#fallbackElement : this.#handle.element;
      this.#fallback = patch(this.#surface, into, this.#fallback, this.#failedContent(next), pass);
      return;
    }
    this.#renderFallback(next, pass);
    this.#handle.update(next, { slots: this.#ownSlots ? {} : this.#patchSlots(next, pass), parent: pass.parent });
  }

  destroy(): void {
    this.#surface.schemes.unfollow(this.#handle);
    this.#handle.destroy();
    for (const slot of this.#slots.values()) for (const item of itemsOf(slot)) this.#destroyItem(item);
    destroyAll(this.#fallback);
  }

  /** The fallback the component receives: a leaf's rendered, or a container's content as plain text. */
  #renderFallback(node: ExpandedComponent, pass: Pass): Fallback {
    if (node.fallback) this.#fallback = patch(this.#surface, this.#fallbackElement, this.#fallback, node.fallback, pass);
    else this.#fallbackElement.textContent = plainText(contentOf(node));
    return { text: plainText(node.fallback ?? contentOf(node)), element: this.#fallbackElement };
  }

  /** What shows when the component can't run (spec §4.5): a leaf's fallback, or a container's slots as Markdown. */
  #failedContent(node: ExpandedComponent): ExpandedNode[] {
    return node.fallback ?? contentOf(node);
  }

  /** Shows the fallback in `box`; a failure that `lasts` frees the slots, and one that doesn't keeps them for later. */
  #showFallback(box: HTMLElement, lasts: boolean, pass: Pass): void {
    if (lasts) {
      for (const slot of this.#slots.values()) for (const item of itemsOf(slot)) this.#destroyItem(item);
      this.#slots.clear();
    }
    if (this.#node.fallback) box.appendChild(this.#fallbackElement);
    else this.#fallback = patch(this.#surface, box, [], this.#failedContent(this.#node), pass);
  }

  /** The component has its box back: a container's content, rendered as its fallback, goes. */
  #hideFallback(): void {
    if (this.#node.fallback) return;
    destroyAll(this.#fallback);
    this.#fallback = [];
  }

  #patchSlots(node: ExpandedComponent, pass: Pass): Slots {
    const contents: [string, { children?: ExpandedNode[]; instances?: ExpandedInstance[] }][] = [];
    if (node.type === "componentContainer") contents.push(["default", node]);
    for (const slot of node.slots ?? []) contents.push([slot.name, slot]);
    const slots: Slots = {};
    for (const [name, content] of contents) {
      const old = this.#slots.get(name);
      if (content.instances) {
        // Every old instance's items are one pool, so an item whose record moved keeps its view.
        const matcher = new Matcher(old ? itemsOf(old) : [], (item: ItemView) => item.view);
        const instances = content.instances.map((instance) => ({ value: instance.value, items: this.#patchItems(matcher, instance.children, pass) }));
        for (const left of matcher.rest()) this.#destroyItem(left);
        this.#slots.set(name, { instances });
        slots[name] = instances.map((instance): SlotInstance => ({ value: instance.value, items: instance.items.map(slotItem) }));
      } else {
        const matcher = new Matcher(old ? itemsOf(old) : [], (item: ItemView) => item.view);
        const items = this.#patchItems(matcher, content.children ?? [], pass);
        for (const left of matcher.rest()) this.#destroyItem(left);
        this.#slots.set(name, { items });
        slots[name] = items.map(slotItem);
      }
    }
    for (const name of [...this.#slots.keys()]) {
      if (contents.some(([present]) => present === name)) continue;
      for (const item of itemsOf(this.#slots.get(name) ?? { items: [] })) this.#destroyItem(item);
      this.#slots.delete(name);
    }
    return slots;
  }

  #patchItems(matcher: Matcher<ItemView>, nodes: readonly ExpandedNode[], pass: Pass): ItemView[] {
    return withHtml(nodes).map((node) => {
      const itemPass: Pass = { ...pass, parent: "block" };
      const old = matcher.take(node, pass);
      const item = old ?? this.#createItem(node, itemPass);
      if (old) {
        old.view.update(node, itemPass);
        old.node = node;
      }
      this.#surface.drawn.add(node, item.view);
      markBox(item.element, [node], "block");
      return item;
    });
  }

  #createItem(node: ExpandedNode, pass: Pass): ItemView {
    const item = this.#surface.document.createElement("div");
    item.className = "compmd-item";
    const view = createView(this.#surface, node, pass);
    item.appendChild(view.dom);
    return { element: item, view, node };
  }

  #destroyItem(item: ItemView): void {
    item.view.destroy();
    item.element.remove();
  }
}
