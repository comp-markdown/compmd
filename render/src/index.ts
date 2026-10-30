// @compmd/render: a CompMD expanded tree (spec §12.3) as a live page, in three layers (README.md).
export { createPage, type Page, type PageOptions } from "./page.ts";
export { createRegion, type Region, type RegionContext, type RegionOptions, type SurfaceOptions } from "./region.ts";
export { mountComponent, type ComponentHandle, type ComponentOptions, type ComponentUpdate } from "./component.ts";
export { rendersSlots, type NodesContext, type NodesMount, type NodesMounted } from "./nodes.ts";
export { Stylesheet } from "./styles.ts";
export { Keys, type EditRange, type KeyOf, type Keyed } from "./keys.ts";
export type { Change, Diagnostic } from "./stable.ts";
export { footnotesOf, definitionsOf, plainText, type Footnotes, type LinkDefinition } from "./document.ts";
export type { CodeRenderer, MathRenderer } from "./render.ts";
export type * from "./contract.ts";
export type * from "./tree.ts";
export { contentOf, isComponent, walk } from "./tree.ts";
