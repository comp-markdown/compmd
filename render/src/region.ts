// The region layer: any list of expanded nodes rendered into a target and updated in place by key, with
// components through the component layer. A page is the region at the root; an editor can put regions in its widgets.
import type { ComponentError, Mount } from "./contract.ts";
import type { LinkDefinition } from "./document.ts";
import type { KeyOf } from "./keys.ts";
import { destroyAll, patch, type CodeRenderer, type MathRenderer, type Surface, type View } from "./render.ts";
import { Stylesheet, type ParentLayout } from "./styles.ts";
import type { ExpandedInput, ExpandedNode } from "./tree.ts";

/** What a page or region needs from its caller. All of it is optional. */
export interface SurfaceOptions {
  /** Host-language components by name: a plain component's default export, or what `react()` or `svelte()` made. */
  components?: Record<string, Mount>;
  /** A component or an input component changed an input; the caller processes the document again with it. */
  onSet?(input: string, value: unknown): void;
  /** A component shows its fallback: it isn't in `components`, or it threw (W118). */
  onError?(error: ComponentError): void;
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

/**
 * Creates a region rendering into `target`, which it owns. Its breakpoints follow the nearest container around it,
 * so a region inside a page follows the page's width.
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
    },
    destroy() {
      destroyAll(views);
      views = [];
      target.replaceChildren();
      if (!options.stylesheet) stylesheet.remove();
    },
  };
}
