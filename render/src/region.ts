// The region layer: any list of expanded nodes rendered into a target and updated in place by key, with
// components through the component layer. A page is the region at the root; an editor can put regions in its widgets.
import type { ComponentError, Mount, RenderError } from "./contract.ts";
import type { LinkDefinition } from "./document.ts";
import type { KeyOf } from "./keys.ts";
import { destroyAll, needsContainer, patch, type CodeRenderer, type MathRenderer, type Surface, type View } from "./render.ts";
import { Stylesheet, type ParentLayout } from "./styles.ts";
import type { ExpandedInput, ExpandedNode } from "./tree.ts";

/** What a page or region needs from its caller. All of it is optional. */
export interface SurfaceOptions {
  /** Host-language components by name: a plain component's default export, or what `react()` or `svelte()` made. */
  components?: Record<string, Mount>;
  /** A component or an input component changed an input; the caller processes the document again with it. */
  onSet?(input: string, value: unknown): void;
  /**
   * A component shows its fallback: it isn't in `components`, or it threw (W118). Or a region's breakpoints have no
   * container to follow (`no-container`), reported once.
   */
  onError?(error: RenderError): void;
  /** What components get as `host` (spec §7.4). */
  host?: unknown;
  /** Renderers for fenced code blocks by language, such as `mermaid`; others show their source. */
  code?: Record<string, CodeRenderer>;
  /** A renderer for math; without one, math shows its TeX source (spec §2.3). */
  math?: MathRenderer;
  /** Where a wiki link points; without an answer it shows its label. */
  wikiLink?(target: string): string | undefined;
}

export interface RegionOptions extends SurfaceOptions {
  /** The document's stylesheet, shared with the page around the region; by default the region has its own. */
  stylesheet?: Stylesheet;
}

/** What a region needs from the document around it. */
export interface RegionContext {
  /** The document's inputs, from the tree's root. */
  inputs?: Record<string, ExpandedInput>;
  /** The level of the last heading before the region (0 for none), for its components' heading level. */
  headingLevel?: number;
  /** The document's footnote numbers. */
  footnotes?: ReadonlyMap<string, number>;
  /** The document's link reference definitions. */
  definitions?: ReadonlyMap<string, LinkDefinition>;
  /** The nodes' keys; without them, nodes match the old views by place. */
  keyOf?: KeyOf;
  /** How the box around the region lays it out (default `block`). */
  parent?: ParentLayout;
}

export interface Region {
  /** Renders `nodes`, or brings what is rendered up to date with them. */
  render(nodes: readonly ExpandedNode[], context?: RegionContext): void;
  destroy(): void;
  readonly stylesheet: Stylesheet;
}

/** The parts of a surface every caller shares, from its options. */
export function surfaceOf(document: Document, stylesheet: Stylesheet, options: SurfaceOptions): Surface {
  return {
    document,
    stylesheet,
    components: options.components ?? {},
    onSet: (input, value) => options.onSet?.(input, value),
    ...(options.onError ? { onError: (error: ComponentError) => options.onError?.(error) } : {}),
    ...(options.host === undefined ? {} : { host: options.host }),
    code: options.code ?? {},
    math: options.math,
    wikiLink: options.wikiLink?.bind(options),
  };
}

/** Whether `element`, or an element around it, is a container that breakpoints can query. */
function insideContainer(element: Element): boolean {
  const view = element.ownerDocument.defaultView;
  for (let at: Element | null = element; at; at = at.parentElement) {
    if (at.classList.contains("compmd-page") || at.classList.contains("compmd-container")) return true;
    const type = view?.getComputedStyle(at).containerType;
    if (type && type !== "normal") return true;
  }
  return false;
}

/**
 * Creates a region rendering into `target`, which it owns. Its breakpoints follow the nearest container around it,
 * which the caller provides: inside a page, the page or a box in it; elsewhere, `target` or an element around it with
 * `compmd-container` (or `container-type: inline-size`). Without one, the region reports `no-container` once.
 *
 * @example
 * const region = createRegion(widget, { components, stylesheet: page.stylesheet });
 * region.render(nodes, { inputs: tree.inputs, headingLevel: 2 });
 */
export function createRegion(target: HTMLElement, options: RegionOptions = {}): Region {
  const document = target.ownerDocument;
  const stylesheet = options.stylesheet ?? new Stylesheet(document);
  const surface = surfaceOf(document, stylesheet, options);
  let views: View[] = [];
  let checked = false;
  // Checked once the target is in the document: an editor's widget is rendered before it is attached.
  const checkContainer = () => {
    if (checked || !target.isConnected) return;
    checked = true;
    if (!insideContainer(target)) {
      options.onError?.({ reason: "no-container", message: "The region's blocks use breakpoints, but no container is around it to follow." });
    }
  };
  return {
    stylesheet,
    render(nodes, context = {}) {
      views = patch(surface, target, views, nodes, {
        inputs: context.inputs ?? {},
        heading: { level: context.headingLevel ?? 0 },
        footnotes: context.footnotes ?? new Map(),
        definitions: context.definitions ?? new Map(),
        keyOf: context.keyOf ?? (() => undefined),
        parent: context.parent ?? "block",
      });
      stylesheet.flush();
      if (checked || !needsContainer(nodes)) return;
      if (target.isConnected) checkContainer();
      else document.defaultView?.requestAnimationFrame(checkContainer);
    },
    destroy() {
      destroyAll(views);
      views = [];
      target.replaceChildren();
      if (!options.stylesheet) stylesheet.remove();
    },
  };
}
