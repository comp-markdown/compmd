// Expanded nodes as DOM, kept up to date: every node has a view, and a new list of nodes is matched to the old views,
// by key where nodes have one (keys come from origins, `keys.ts`) and by place among nodes of the same shape where they
// don't. A matched view updates in place, so components keep their instances and the DOM keeps focus and selection.
import { mountComponent, type ComponentHandle } from "./component.ts";
import type { ComponentError, Fallback, Mount, SlotInstance, SlotItem, Slots } from "./contract.ts";
import { applyBox } from "./box.ts";
import { plainText, type LinkDefinition } from "./document.ts";
import { safeUrl, withHtml } from "./html.ts";
import { nodesMountOf, runnable } from "./nodes.ts";
import { fills, layoutOf, ownParams, usesBreakpoints, type ParentLayout, type Stylesheet } from "./styles.ts";
import { contentOf, isComponent, rendererOf, textOf, type ExpandedCode, type ExpandedComponent, type ExpandedInput, type ExpandedInstance, type ExpandedNode, type MarkdownNode } from "./tree.ts";

/** Renders a fenced code block of one language, such as a Mermaid diagram, into `target`. */
export type CodeRenderer = (source: string, target: HTMLElement) => void;
/** Renders TeX math into `target`. */
export type MathRenderer = (tex: string, options: { display: boolean }, target: HTMLElement) => void;

/** What every view of one document shares: where it renders, the components, and the caller's callbacks. */
export interface Surface {
  document: Document;
  stylesheet: Stylesheet;
  components: Readonly<Record<string, Mount>>;
  onSet: (input: string, value: unknown) => void;
  onError?: ((error: ComponentError) => void) | undefined;
  host?: unknown;
  code: Readonly<Record<string, CodeRenderer>>;
  math?: MathRenderer | undefined;
  /** Where a wiki link points (spec §2.3); without an answer it renders its label as text. */
  wikiLink?: ((target: string) => string | undefined) | undefined;
}

/** What one render carries in document order. */
export interface Pass {
  inputs: Readonly<Record<string, ExpandedInput>>;
  /** The level of the last heading rendered so far (0 before any), for components' heading level (spec §7.2). */
  heading: { level: number };
  footnotes: ReadonlyMap<string, number>;
  definitions: ReadonlyMap<string, LinkDefinition>;
  keyOf(node: ExpandedNode): string | undefined;
  /** How the box around the current list lays it out. */
  parent: ParentLayout;
}

/** A rendered node. */
export interface View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: Node;
  update(node: ExpandedNode, pass: Pass): void;
  destroy(): void;
}

/** A node made by the renderer: an element with a tag, attributes, and text or children. */
interface ElementNode extends MarkdownNode {
  type: "compmdElement";
  tag: string;
  attributes?: Record<string, string>;
  text?: string;
}

const element = (tag: string, content: Omit<ElementNode, "type" | "tag"> = {}): ElementNode => ({ type: "compmdElement", tag, ...content });

const INPUTS: Record<string, string> = { "date-input": "date", "number-input": "number", "text-input": "text", "select-input": "select", "toggle-input": "checkbox" };

/** What a view is, so a node is matched only to a view that can show it. */
export function shapeOf(node: ExpandedNode): string {
  if (isComponent(node)) return `${node.type}:${node.name}`;
  const { type } = node;
  if (type === "heading") return `heading:${textOf(node.depth)}`;
  if (type === "list") return `list:${String(node.ordered === true)}`;
  if (type === "code") {
    const renderer = rendererOf(node);
    return renderer ? `code:${textOf(node.lang)}:${renderer.name}` : `code:${textOf(node.lang)}`;
  }
  if (type === "compmdElement" || type === "compmdHtml") return `${type}:${textOf(node.tag)}`;
  return type;
}

// ── matching ─────────────────────────────────────────────────────────────────

/**
 * Old entries (views, or slot items) to match new nodes to. Each entry is taken at most once: a keyed node takes the
 * entry with its key and shape, any other node the next unkeyed entry of its shape.
 */
export class Matcher<T> {
  readonly #keyed = new Map<string, T[]>();
  readonly #unkeyed = new Map<string, T[]>();

  constructor(entries: readonly T[], view: (entry: T) => View) {
    for (const entry of entries) {
      const { key, shape } = view(entry);
      const into = key === undefined ? this.#unkeyed : this.#keyed;
      const id = key === undefined ? shape : `${key}\u0000${shape}`;
      into.set(id, [...(into.get(id) ?? []), entry]);
    }
  }

  take(node: ExpandedNode, pass: Pass): T | undefined {
    const key = pass.keyOf(node);
    return key === undefined ? this.#unkeyed.get(shapeOf(node))?.shift() : this.#keyed.get(`${key}\u0000${shapeOf(node)}`)?.shift();
  }

  /** The entries no node took. */
  rest(): T[] {
    return [...this.#keyed.values(), ...this.#unkeyed.values()].flat();
  }
}

/** Puts `nodes` in `parent` in order, moving only those out of place: a moved element loses its focus. */
function arrange(parent: Node, nodes: readonly Node[]): void {
  let expected: ChildNode | null = parent.firstChild;
  for (const node of nodes) {
    if (node === expected) expected = expected.nextSibling;
    else parent.insertBefore(node, expected);
  }
}

/** Views for `nodes`: the old views `matcher` holds where they match, updated, and new ones for the rest. */
export function viewsFor(surface: Surface, matcher: Matcher<View>, nodes: readonly ExpandedNode[], pass: Pass): View[] {
  return nodes.map((node) => {
    const old = matcher.take(node, pass);
    if (!old) return createView(surface, node, pass);
    old.update(node, pass);
    return old;
  });
}

/** Takes the views out of the document and destroys them. */
export function discard(views: readonly View[]): void {
  for (const view of views) {
    view.dom.parentNode?.removeChild(view.dom);
    view.destroy();
  }
}

/**
 * Renders `nodes` into `parent`, which holds only these views, reusing `views` where they match.
 *
 * @example
 * let views = patch(surface, element, [], nodes, pass);
 * views = patch(surface, element, views, nextNodes, nextPass);
 */
export function patch(surface: Surface, parent: Node, views: readonly View[], nodes: readonly ExpandedNode[], pass: Pass): View[] {
  const matcher = new Matcher(views, (view) => view);
  const next = viewsFor(surface, matcher, withHtml(nodes), pass);
  discard(matcher.rest());
  arrange(parent, next.map((view) => view.dom));
  return next;
}

export function destroyAll(views: readonly View[]): void {
  for (const view of views) view.destroy();
}

// ── views ────────────────────────────────────────────────────────────────────

export function createView(surface: Surface, node: ExpandedNode, pass: Pass): View {
  const key = pass.keyOf(node);
  if (isComponent(node)) {
    if (node.name === "box") return new BoxView(surface, node, pass, key);
    if (node.name in INPUTS) return new InputView(surface, node, pass, key);
    return new ComponentView(surface, node, pass, key);
  }
  if (node.type === "text") return textView(surface, node, key);
  const renderer = rendererOf(node);
  if (renderer) return new RenderedCodeView(surface, node, renderer, pass, key);
  if ((node.type === "code" && typeof node.lang === "string" && surface.code[node.lang]) || ((node.type === "math" || node.type === "inlineMath") && surface.math)) {
    return new CallbackView(surface, node, key);
  }
  return new ElementView(surface, node, pass, key);
}

function textView(surface: Surface, node: MarkdownNode, key: string | undefined): View {
  const dom = surface.document.createTextNode(textOf(node.value));
  return {
    shape: "text",
    key,
    dom,
    update: (next) => {
      const value = textOf((next as MarkdownNode).value);
      if (dom.data !== value) dom.data = value;
    },
    destroy: () => undefined,
  };
}

/** How a Markdown node renders: its element, and either its text or the nodes inside it. */
interface Rendering {
  tag: string;
  attributes: Record<string, string>;
  text?: string;
  children?: readonly ExpandedNode[];
}

const ALERT_TITLES: Record<string, string> = { note: "Note", tip: "Tip", important: "Important", warning: "Warning", caution: "Caution" };

function linkOrText(url: string, title: unknown, children: readonly ExpandedNode[]): Rendering {
  const href = safeUrl(url);
  if (href === undefined) return { tag: "span", attributes: {}, children };
  return { tag: "a", attributes: { href, ...(typeof title === "string" && title ? { title } : {}) }, children };
}

function image(url: string, alt: unknown, title: unknown): Rendering {
  const src = safeUrl(url);
  return { tag: "img", attributes: { ...(src === undefined ? {} : { src }), alt: textOf(alt), ...(typeof title === "string" && title ? { title } : {}) } };
}

function table(node: MarkdownNode): Rendering {
  const align = (node.align as (string | null)[] | undefined) ?? [];
  const rows = node.children ?? [];
  const row = (tr: ExpandedNode, cell: string) =>
    element("tr", {
      children: (tr.children ?? []).map((td, i) => element(cell, { ...(align[i] ? { attributes: { class: `compmd-align-${align[i] ?? ""}` } } : {}), children: td.children ?? [] })),
    });
  const [head, ...body] = rows;
  return { tag: "table", attributes: {}, children: [element("thead", { children: head ? [row(head, "th")] : [] }), element("tbody", { children: body.map((tr) => row(tr, "td")) })] };
}

/** The rendering of a Markdown node (CommonMark and GFM, with alerts and math; spec §2.3). */
function renderingOf(surface: Surface, node: MarkdownNode, pass: Pass): Rendering {
  const children = node.children ?? [];
  const plain = (tag: string, attributes: Record<string, string> = {}): Rendering => ({ tag, attributes, children });
  switch (node.type) {
    case "paragraph":
      return plain("p");
    case "heading":
      return plain(`h${textOf(node.depth)}`);
    case "emphasis":
      return plain("em");
    case "strong":
      return plain("strong");
    case "delete":
      return plain("del");
    case "inlineCode":
      return { tag: "code", attributes: {}, text: textOf(node.value) };
    case "break":
      return { tag: "br", attributes: {} };
    case "thematicBreak":
      return { tag: "hr", attributes: {} };
    case "blockquote": {
      const type = typeof node.alert === "string" ? node.alert : undefined;
      if (type === undefined) return plain("blockquote", { class: "compmd-quote" });
      const title = element("p", { attributes: { class: "compmd-alert-title" }, text: ALERT_TITLES[type] ?? type });
      return { tag: "div", attributes: { class: `compmd-alert compmd-alert-${type}`, role: "note" }, children: [title, ...children] };
    }
    case "list": {
      const start = typeof node.start === "number" && node.start !== 1 && node.ordered === true ? { start: String(node.start) } : {};
      // A tight list's items hold their paragraphs without margins, as HTML renders them without `<p>` (CommonMark §5.3).
      const tight = node.spread === false ? { class: "compmd-tight" } : {};
      return plain(node.ordered === true ? "ol" : "ul", { ...start, ...tight });
    }
    case "listItem": {
      if (typeof node.checked !== "boolean") return plain("li");
      const box = element("input", { attributes: { type: "checkbox", disabled: "", ...(node.checked ? { checked: "" } : {}) } });
      return { tag: "li", attributes: { class: "compmd-task" }, children: [box, ...children] };
    }
    case "link":
      return linkOrText(textOf(node.url), node.title, children);
    case "image":
      return image(textOf(node.url), node.alt, node.title);
    case "linkReference": {
      const definition = pass.definitions.get(textOf(node.identifier));
      return definition ? linkOrText(definition.url, definition.title, children) : plain("span");
    }
    case "imageReference": {
      const definition = pass.definitions.get(textOf(node.identifier));
      return definition ? image(definition.url, node.alt, definition.title) : { tag: "span", attributes: {}, text: textOf(node.alt) };
    }
    case "table":
      return table(node);
    case "code":
      return { tag: "pre", attributes: {}, children: [element("code", { ...(node.lang ? { attributes: { class: `language-${textOf(node.lang)}` } } : {}), text: textOf(node.value) })] };
    case "math":
      return { tag: "pre", attributes: {}, children: [element("code", { attributes: { class: "language-math math-display" }, text: textOf(node.value) })] };
    case "inlineMath":
      return { tag: "code", attributes: { class: "language-math math-inline" }, text: textOf(node.value) };
    case "footnoteReference": {
      const number = String(pass.footnotes.get(textOf(node.identifier)) ?? "?");
      return { tag: "sup", attributes: {}, children: [element("a", { attributes: { href: `#compmd-fn-${number}`, id: `compmd-fnref-${number}` }, text: number })] };
    }
    case "wikiLink": {
      const label = textOf(node.label ?? node.target);
      const href = surface.wikiLink?.(textOf(node.target));
      const safe = href === undefined ? undefined : safeUrl(href);
      return safe === undefined ? { tag: "span", attributes: {}, text: label } : { tag: "a", attributes: { href: safe }, text: label };
    }
    case "compmdElement":
    case "compmdHtml": {
      const own = node as unknown as ElementNode;
      return { tag: own.tag, attributes: own.attributes ?? {}, ...(own.text === undefined ? { children: own.children ?? [] } : { text: own.text }) };
    }
    case "definition":
    case "footnoteDefinition":
      // A definition shows where it is used; the page lists footnotes at the end.
      return { tag: "template", attributes: {} };
    default:
      return plain("span");
  }
}

class ElementView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  #attributes: Record<string, string> = {};
  #children: View[] = [];

  constructor(surface: Surface, node: MarkdownNode, pass: Pass, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    this.dom = surface.document.createElement(renderingOf(surface, node, pass).tag);
    this.update(node, pass);
  }

  update(node: ExpandedNode, pass: Pass): void {
    const rendering = renderingOf(this.#surface, node as MarkdownNode, pass);
    for (const name of Object.keys(this.#attributes)) if (!(name in rendering.attributes)) this.dom.removeAttribute(name);
    for (const [name, value] of Object.entries(rendering.attributes)) if (this.dom.getAttribute(name) !== value) this.dom.setAttribute(name, value);
    this.#attributes = rendering.attributes;
    if (node.type === "heading") pass.heading.level = Number(node.depth);
    if (rendering.text !== undefined) {
      if (this.dom.textContent !== rendering.text) this.dom.textContent = rendering.text;
    } else if (rendering.children) {
      this.#children = patch(this.#surface, this.dom, this.#children, rendering.children, { ...pass, parent: "block" });
    }
  }

  destroy(): void {
    destroyAll(this.#children);
  }
}

/** A code block or math that the caller's renderer draws; if it throws, the source shows instead. */
class CallbackView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly dom: HTMLElement;
  readonly #surface: Surface;
  #source: string | undefined;

  constructor(surface: Surface, node: MarkdownNode, key: string | undefined) {
    this.#surface = surface;
    this.shape = shapeOf(node);
    this.key = key;
    const inline = node.type === "inlineMath";
    this.dom = surface.document.createElement(inline ? "span" : "div");
    this.dom.className = node.type === "code" ? `compmd-code language-${textOf(node.lang)}` : `compmd-math ${inline ? "math-inline" : "math-display"}`;
    this.update(node);
  }

  update(next: ExpandedNode): void {
    const node = next as MarkdownNode;
    const source = textOf(node.value);
    if (source === this.#source) return;
    this.#source = source;
    this.dom.replaceChildren();
    try {
      if (node.type === "code") this.#surface.code[textOf(node.lang)]?.(source, this.dom);
      else this.#surface.math?.(source, { display: node.type === "math" }, this.dom);
    } catch {
      this.dom.replaceChildren(this.#surface.document.createTextNode(source));
    }
  }

  destroy(): void {
    return undefined;
  }
}

/**
 * A code block a code renderer draws (spec §7.7): the renderer's component in its box, the block's text among its
 * params as `code` and `alt` as its fallback text; the code block is its fallback.
 */
class RenderedCodeView implements View {
  readonly shape: string;
  readonly key: string | undefined;
  readonly #handle: ComponentHandle;
  readonly #code: View;
  #current: { node: ExpandedComponent; inputs: Pass["inputs"] };

  constructor(surface: Surface, node: MarkdownNode, renderer: ExpandedComponent, pass: Pass, key: string | undefined) {
    this.shape = shapeOf(node);
    this.key = key;
    this.#current = { node: renderer, inputs: pass.inputs };
    this.#code = new ElementView(surface, plainCode(node), pass, undefined);
    const fallback = surface.document.createElement("div");
    fallback.appendChild(this.#code.dom);
    // The component layer appends the box to its target; the region places it, so it starts in a fragment.
    this.#handle = mountComponent(surface.document.createDocumentFragment(), renderer, {
      mount: runnable(surface.components[renderer.name], () => this.#current),
      slots: {},
      fallback: { text: textOf((node as ExpandedCode).alt), element: fallback },
      headingLevel: Math.min(6, pass.heading.level + 1),
      onSet: surface.onSet,
      ...(surface.onError ? { onError: surface.onError } : {}),
      host: surface.host,
      stylesheet: surface.stylesheet,
      parent: pass.parent,
    });
  }

  get dom(): Node {
    return this.#handle.element;
  }

  update(node: ExpandedNode, pass: Pass): void {
    this.#code.update(plainCode(node), pass);
    const renderer = rendererOf(node);
    if (!renderer) return;
    this.#current = { node: renderer, inputs: pass.inputs };
    this.#handle.update(renderer, { slots: {}, parent: pass.parent });
  }

  destroy(): void {
    this.#handle.destroy();
    this.#code.destroy();
  }
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
class BoxView implements View {
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
class InputView implements View {
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
  return isComponent(node) ? { element, component: { name: node.name, params: ownParams(node.params) } } : { element };
}

class ComponentView implements View {
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
    const fallback = this.#renderFallback(node, pass);
    const slots = this.#ownSlots ? {} : this.#patchSlots(node, pass);
    // The component layer appends the box to its target; the region places it, so it starts in a fragment.
    this.#handle = mountComponent(surface.document.createDocumentFragment(), node, {
      mount: runnable(mount, () => ({ node: this.#node, inputs: this.#inputs })),
      slots,
      fallback,
      headingLevel,
      onSet: surface.onSet,
      ...(surface.onError ? { onError: surface.onError } : {}),
      showFallback: (box) => {
        this.#showFallback(box, pass);
      },
      host: surface.host,
      stylesheet: surface.stylesheet,
      parent: pass.parent,
    });
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

  #showFallback(box: HTMLElement, pass: Pass): void {
    for (const slot of this.#slots.values()) for (const item of itemsOf(slot)) this.#destroyItem(item);
    this.#slots.clear();
    if (this.#node.fallback) box.appendChild(this.#fallbackElement);
    else this.#fallback = patch(this.#surface, box, [], this.#failedContent(this.#node), pass);
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
