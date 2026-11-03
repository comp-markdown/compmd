// Where the page's DOM came from in the source, and back: every view drawn for a node with an origin (spec §12.3) is
// kept with that node, so the page can say which range made an element and which elements a range drew. The page
// gives the ranges as they are in the current text: after a splice, the origins of the blocks it kept are behind it.
import type { View } from "./render.ts";
import { walk, type ExpandedNode, type Origin, type Point } from "./tree.ts";

/**
 * A range of the current text: `start` and `end` points as origins have them (spec §12.3), lines and columns 1-based,
 * offsets 0-based, columns and offsets in code points. A point has an offset when the processor gave origins one.
 */
export interface Source {
  start: Point;
  end: Point;
}

/** What a node the renderer made draws for, when it stands for a node of the tree: a footnote's item, its definition. */
export interface DrawnFor {
  drawnFor?: ExpandedNode;
}

/** The views drawn for each node with an origin. */
export class Drawn {
  readonly #views = new WeakMap<ExpandedNode, View[]>();

  /** Keeps `view` as drawn for `node`, or for the node it stands for. */
  add(node: ExpandedNode, view: View): void {
    const source = (node as DrawnFor).drawnFor ?? node;
    // A definition's own view is an empty `template`: it shows where it is used.
    if (!source.origin || view.dom.nodeName === "TEMPLATE") return;
    const views = this.#views.get(source);
    if (!views) this.#views.set(source, [view]);
    else if (!views.includes(view)) views.push(view);
  }

  /** What was drawn for `node` that is in `root` now: a component's fallback shows only while it fails. */
  of(node: ExpandedNode, root: Node): Node[] {
    return (this.#views.get(node) ?? []).map((view) => view.dom).filter((dom) => inside(root, dom));
  }
}

/** One node's DOM, with its range in the current text. */
export interface Placed {
  dom: Node;
  source: Source;
}

/**
 * What `nodes` drew that is in `root` now, each with its range: what `sourceOf` makes of its node's origin. A node
 * without an origin has nothing of its own: its DOM belongs to the nearest node around it that has one.
 */
export function placedOf(nodes: readonly ExpandedNode[], drawn: Drawn, root: Node, sourceOf: (origin: Origin) => Source): Placed[] {
  const placed: Placed[] = [];
  walk(nodes, (node) => {
    if (!node.origin) return;
    const source = sourceOf(node.origin);
    for (const dom of drawn.of(node, root)) placed.push({ dom, source });
  });
  return placed;
}

/** `point`, `lines` lines and `shift` code points on. */
export function moved(point: Point, lines: number, shift: number): Point {
  return { line: point.line + lines, column: point.column, ...(point.offset === undefined ? {} : { offset: point.offset + shift }) };
}

// By node type, not `instanceof`: a page can render into another frame's document.
const isElement = (node: Node): node is Element => node.nodeType === Node.ELEMENT_NODE;

/** The parent of `node`, through a shadow root to its host. */
function parentOf(node: Node): Node | null {
  const parent = node.parentNode;
  return parent && "host" in parent ? (parent as ShadowRoot).host : parent;
}

/** Whether `node` is `root` or inside it, a component's shadow roots included. */
function inside(root: Node, node: Node): boolean {
  for (let at: Node | null = node; at; at = parentOf(at)) if (at === root) return true;
  return false;
}

/**
 * The range that made `target`: its own node's, or the nearest node's around it, up to `root`. Of DOM drawn for several
 * nodes (a component's box and its definition's body), the innermost node's.
 */
export function sourceIn(placed: Iterable<Placed>, target: Node, root: Node): Source | undefined {
  const sources = new Map<Node, Source>();
  // In walk order, an inner node after the nodes around it.
  for (const { dom, source } of placed) sources.set(dom, source);
  for (let at: Node | null = target; at && at !== root; at = parentOf(at)) {
    const source = sources.get(at);
    if (source) return source;
  }
  return undefined;
}

/** How long {@link reveal} marks what it scrolled to, in ms. */
export const REVEALED_MS = 1500;

/** When each element marked now loses its mark: a second reveal starts its time again. */
const marks = new WeakMap<Element, ReturnType<typeof setTimeout>>();

/**
 * Scrolls to `elements`, as little as shows the first, and marks each with the class `compmd-revealed` for
 * {@link REVEALED_MS}: an outline in the theme's accent (`--compmd-accent`), which a host may style otherwise.
 *
 * @param elements - What to show, such as what a page's `elementsAt` gives for an editor's caret.
 *
 * @example
 * reveal(page.elementsAt(codePointOffset(text, caret)));
 */
export function reveal(elements: readonly Element[]): void {
  elements[0]?.scrollIntoView({ block: "nearest", inline: "nearest" });
  for (const element of elements) {
    clearTimeout(marks.get(element));
    element.classList.add("compmd-revealed");
    marks.set(
      element,
      setTimeout(() => {
        element.classList.remove("compmd-revealed");
        marks.delete(element);
      }, REVEALED_MS),
    );
  }
}

const length = ({ start, end }: Source): number => (end.offset ?? 0) - (start.offset ?? 0);

/**
 * The elements drawn for the innermost range that holds `offset`. A range holds the offsets from its start to its end;
 * where one range ends and the next starts, the one that starts there. Several elements when the range drew several
 * (an `each`'s iterations), the outermost of those it drew inside one another (a box and its body).
 */
export function elementsIn(placed: Iterable<Placed>, offset: number): Element[] {
  const holding = [...placed].filter(({ dom, source }) => isElement(dom) && source.start.offset !== undefined && source.end.offset !== undefined && source.start.offset <= offset && offset <= source.end.offset);
  const starting = holding.filter(({ source }) => offset < (source.end.offset ?? 0) || length(source) === 0);
  const candidates = starting.length > 0 ? starting : holding;
  const shortest = Math.min(...candidates.map(({ source }) => length(source)));
  const elements = candidates.filter(({ source }) => length(source) === shortest).map(({ dom }) => dom as Element);
  return elements.filter((element) => !elements.some((other) => other !== element && other.contains(element)));
}
