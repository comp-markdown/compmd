// Expanded nodes as DOM, kept up to date: every node has a view, and a new list of nodes is matched to the old views,
// by key where nodes have one (keys come from origins, `keys.ts`) and by place among nodes of the same shape where they
// don't. A matched view updates in place, so components keep their instances and the DOM keeps focus and selection.
// The views themselves are in `markdown-views.ts` and `component-views.ts`.
import type { Busy } from "./busy.ts";
import { BoxView, ComponentView, INPUTS, InputView, RenderedCodeView } from "./component-views.ts";
import type { ComponentError, ComponentRecovered, Mount } from "./contract.ts";
import type { LinkDefinition } from "./document.ts";
import { withHtml } from "./html.ts";
import type { ImageSource } from "./images.ts";
import { CallbackView, CodeBlockView, ElementView, ImageView, isImage, textView } from "./markdown-views.ts";
import type { Schemes } from "./scheme.ts";
import type { Drawn } from "./source.ts";
import type { ParentLayout, Stylesheet } from "./styles.ts";
import { isComponent, rendererOf, textOf, type ExpandedInput, type ExpandedNode } from "./tree.ts";

/** Renders a fenced code block of one language, such as a Mermaid diagram, into `target`. */
export type CodeRenderer = (source: string, target: HTMLElement) => void;
/** Renders TeX math into `target`. */
export type MathRenderer = (tex: string, options: { display: boolean }, target: HTMLElement) => void;
/**
 * Highlights a code block that shows as code: `code`, written in the fence language `language` (`""` for none), into
 * `target`, the block's `<code>`, which holds `code` as text until then. It may do so later, once what it needs has
 * loaded; it is called again only when the block's text or language changes.
 */
export type CodeHighlighter = (code: string, language: string, target: HTMLElement) => void;

/**
 * The mounts of the components a render made, waiting until it has placed their boxes in the document and flushed the
 * stylesheet: a component then mounts into a box with its styles, its theme tokens and its place. They run in
 * document order, a component before those in its slots, whose items it places in its own DOM as it mounts.
 */
export class PendingMounts {
  #mounts: { run?: () => void }[] = [];

  /** Keeps a place in the order now, for a mount given later: a component's, before its slots are rendered. */
  reserve(): (mount: () => void) => void {
    const place: { run?: () => void } = {};
    this.#mounts.push(place);
    return (mount) => {
      place.run = mount;
    };
  }

  /** Keeps `mount` for {@link run}, after those already kept. */
  readonly later = (mount: () => void): void => {
    this.reserve()(mount);
  };

  /** Runs the mounts waiting, in their order. */
  run(): void {
    const mounts = this.#mounts;
    this.#mounts = [];
    for (const { run } of mounts) run?.();
  }
}

/** What every view of one document shares: where it renders, the components, and the caller's callbacks. */
export interface Surface {
  document: Document;
  stylesheet: Stylesheet;
  /** The mounts of the components the current render made; the page or region runs them once it has placed them. */
  pending: PendingMounts;
  /** The colour scheme components are shown in, which the components it gives follow. */
  schemes: Schemes;
  components: Readonly<Record<string, Mount>>;
  onSet: (input: string, value: unknown) => void;
  onError?: ((error: ComponentError) => void) | undefined;
  onRecovered?: ((recovered: ComponentRecovered) => void) | undefined;
  host?: unknown;
  code: Readonly<Record<string, CodeRenderer>>;
  math?: MathRenderer | undefined;
  /** Highlights the code blocks that show as code; without it, their text is plain. */
  highlight?: CodeHighlighter | undefined;
  /** Where a wiki link points (spec §2.3); without an answer it renders its label as text. */
  wikiLink?: ((target: string) => string | undefined) | undefined;
  /** Where images come from; without it, each loads from its `src`. */
  images?: ImageSource | undefined;
  /** The views drawn for each node with an origin. */
  drawn: Drawn;
  /** What the components are still drawing (`context.busy`). */
  busy: Busy;
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
  if (type === "componentPending") return `${type}:${pendingTag(node.children ?? [])}`;
  return type;
}

/** Nodes of inline content: what a paragraph holds. */
const INLINE = new Set(["text", "emphasis", "strong", "delete", "inlineCode", "inlineMath", "link", "linkReference", "image", "imageReference", "break", "footnoteReference", "wikiLink", "html", "componentInline", "componentPending"]);

/** The element a pending use is drawn in: a `span` in inline content, as an inline component's fallback is, else a `div`. */
export function pendingTag(children: readonly ExpandedNode[]): "span" | "div" {
  return children.every((child) => INLINE.has(child.type)) ? "span" : "div";
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
    const view = old ?? createView(surface, node, pass);
    old?.update(node, pass);
    surface.drawn.add(node, view);
    return view;
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

/** A new view of `node`: of its component, its code renderer, the caller's renderer for it, or its Markdown. */
export function createView(surface: Surface, node: ExpandedNode, pass: Pass): View {
  const key = pass.keyOf(node);
  if (isComponent(node)) {
    if (node.name === "box") return new BoxView(surface, node, pass, key);
    if (node.name in INPUTS) return new InputView(surface, node, pass, key);
    return new ComponentView(surface, node, pass, key);
  }
  if (node.type === "text") return textView(surface, node, key);
  if (surface.images && isImage(node)) return new ImageView(surface, surface.images, node, pass, key);
  const renderer = rendererOf(node);
  if (renderer) return new RenderedCodeView(surface, node, renderer, pass, key);
  if ((node.type === "code" && typeof node.lang === "string" && surface.code[node.lang]) || ((node.type === "math" || node.type === "inlineMath") && surface.math)) {
    return new CallbackView(surface, node, key);
  }
  if (node.type === "code") return new CodeBlockView(surface, node, key);
  return new ElementView(surface, node, pass, key);
}
