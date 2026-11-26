// Each block of the page with the line it starts on (docs/editor-support.md, decision 8): every top-level element the
// renderer draws carries `data-line`, its line in the current text, so a page that follows an editor (the preview)
// scrolls to the block being edited. A block moved by a change before it is marked again, the same element.
import type { Placed } from "./source.ts";

/** `Node.ELEMENT_NODE`, which a DOM made outside a browser (happy-dom's, for a build) doesn't put on the global. */
const ELEMENT_NODE = 1;

/**
 * Marks each element of `placed` that is a child of `root` with the line its range starts on, `data-line`: of DOM drawn
 * for several nodes (a component's box and its body), the outermost node's, which `placed` has first.
 */
export function markBlockLines(placed: Iterable<Placed>, root: Node): void {
  const marked = new Set<Node>();
  for (const { dom, source } of placed) {
    if (dom.parentNode !== root || dom.nodeType !== ELEMENT_NODE || marked.has(dom)) continue;
    marked.add(dom);
    const line = String(source.start.line);
    const element = dom as Element;
    if (element.getAttribute("data-line") !== line) element.setAttribute("data-line", line);
  }
}

/** The block of `root` that holds `line`: the last of its children marked with a line at or before it. */
export function blockAt(root: Element, line: number): Element | undefined {
  let found: Element | undefined;
  for (const child of root.children) {
    const marked = Number(child.getAttribute("data-line"));
    if (!child.hasAttribute("data-line") || Number.isNaN(marked)) continue;
    if (marked > line) break;
    found = child;
  }
  return found;
}

/**
 * Scrolls, as little as it takes, to the block of `root` that holds `line`, without marking it: for a page that follows
 * an editor as the writer types, where `reveal`'s mark would flash at every key.
 */
export function scrollToLine(root: Element, line: number): void {
  blockAt(root, line)?.scrollIntoView({ block: "nearest", inline: "nearest" });
}
