// A use's fallback for paper (spec §7.3, §14): for a component whose signature says `print: fallback`, its box and its
// fallback side by side in a wrapper that doesn't box them (`display: contents`); the print styles show the fallback
// and hide the box on paper, and the screen shows the box alone.
import type { Pass, Surface, View } from "./render.ts";
import { destroyAll, patch } from "./render.ts";
import type { ExpandedNode } from "./tree.ts";

/**
 * The wrapper around a use's box, with its fallback for paper, rendered as any other nodes are.
 *
 * @example
 * const paper = new PaperFallback(surface, box, false);
 * paper.render(contentOf(node), pass); // a container's slots as Markdown
 * region.append(paper.element);
 */
export class PaperFallback {
  /** The wrapper: the box, then the fallback for paper. */
  readonly element: HTMLElement;
  readonly #surface: Surface;
  readonly #paper: HTMLElement;
  #views: View[] = [];

  /** Wraps `box`; `inline` for an inline component, whose fallback flows in its text. */
  constructor(surface: Surface, box: Node, inline: boolean) {
    this.#surface = surface;
    const tag = inline ? "span" : "div";
    this.element = surface.document.createElement(tag);
    this.element.className = "compmd-print-wrap";
    this.#paper = surface.document.createElement(tag);
    this.#paper.className = "compmd-print-paper";
    this.element.append(box, this.#paper);
  }

  /** Renders `nodes`, the use's fallback (spec §4.5), for paper. */
  render(nodes: readonly ExpandedNode[], pass: Pass): void {
    this.#views = patch(this.#surface, this.#paper, this.#views, nodes, pass);
  }

  destroy(): void {
    destroyAll(this.#views);
    this.#views = [];
  }
}
