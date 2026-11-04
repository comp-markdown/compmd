// @compmd/render: a CompMD expanded tree (spec §12.3) as a live page, in three layers (README.md).
export { createPage, type Page, type PageOptions } from "./page.ts";
export { createRegion, type Region, type RegionContext, type RegionOptions, type SurfaceOptions } from "./region.ts";
export { mountComponent, type ComponentHandle, type ComponentOptions, type ComponentUpdate } from "./component.ts";
export { rendersSlots, type NodesContext, type NodesMount, type NodesMounted } from "./nodes.ts";
export { Stylesheet } from "./styles.ts";
export { schemeOf } from "./scheme.ts";
export { Keys, codePointOffset, utf16Offset, type EditRange, type KeyOf, type Keyed } from "./keys.ts";
export { reveal, REVEALED_MS, type Source } from "./source.ts";
export type { Line, Rect } from "./lines.ts";
export { BUSY_MS } from "./busy.ts";
export type { Change, Diagnostic } from "./stable.ts";
export { footnotesOf, definitionsOf, plainText, type Footnotes, type LinkDefinition } from "./document.ts";
export type { CodeHighlighter, CodeRenderer, MathRenderer } from "./render.ts";
export { imageSources, type ImageSource } from "./images.ts";
export type * from "./contract.ts";
export type * from "./tree.ts";
export { contentOf, isComponent, rendererOf, walk } from "./tree.ts";
