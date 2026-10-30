// The expanded tree (spec §12.3), the renderer's whole input, as JSON holds it (§12.4).

/** A place in the document: lines and columns 1-based, the offset 0-based; columns and offsets count code points. */
export interface Point {
  line: number;
  column: number;
  offset?: number;
}

/** The range of the processed document that made a node, and how many nodes before it in its list have that range. */
export interface Origin {
  start: Point;
  end: Point;
  occurrence: number;
}

/** An evaluated param. One bound to an input names it in `input`, and `value` is the input's current value. */
export interface ExpandedParam {
  key: string;
  breakpoint: string | null;
  value: unknown;
  input?: string;
}

/** A template slot's element, with the slot content expanded for it (spec §7.5). */
export interface ExpandedInstance {
  value: unknown;
  children: ExpandedNode[];
}

/** A named slot of a component. */
export interface ExpandedSlot {
  type: "componentSlot";
  name: string;
  children: ExpandedNode[];
  instances?: ExpandedInstance[];
  origin?: Origin;
}

/** A component the renderer renders itself: `box`, an input component or a host-language component. */
export interface ExpandedComponent {
  type: "componentContainer" | "componentLeaf" | "componentInline";
  name: string;
  params: ExpandedParam[];
  /** A container's default slot. */
  children?: ExpandedNode[];
  /** A container's default slot when it is a template slot. */
  instances?: ExpandedInstance[];
  slots?: ExpandedSlot[];
  /** A leaf's or an inline component's fallback (inline content). */
  fallback?: ExpandedNode[];
  origin?: Origin;
}

/** Any mdast node (https://github.com/syntax-tree/mdast), with CompMD's additions (spec §2.3, §12.2). */
export interface MarkdownNode {
  type: string;
  children?: ExpandedNode[];
  origin?: Origin;
  [field: string]: unknown;
}

/** A node in a list of blocks or inline content. (Named slots are only ever in a component's `slots`.) */
export type ExpandedNode = ExpandedComponent | MarkdownNode;

/** A code block; one a code renderer renders (spec §7.7) names it in `component`, with its params, and has its `alt`. */
export interface ExpandedCode extends MarkdownNode {
  type: "code";
  lang: string | null;
  meta: string | null;
  value: string;
  component?: { name: string; params: ExpandedParam[] };
  alt?: string;
}

/** The leaf of the code renderer that renders `node`, a code block, as it is mounted; `undefined` when none does. */
export function rendererOf(node: ExpandedNode): ExpandedComponent | undefined {
  if (node.type !== "code") return undefined;
  const { component, origin } = node as ExpandedCode;
  if (!component) return undefined;
  return { type: "componentLeaf", name: component.name, params: component.params, ...(origin ? { origin } : {}) };
}

/** One of the document's inputs, as the root lists it. */
export interface ExpandedInput {
  type: string;
  members?: string[];
  value: unknown;
}

export interface ExpandedRoot {
  type: "root";
  children: ExpandedNode[];
  inputs?: Record<string, ExpandedInput>;
}

/** An edit in code points (spec §10.5): the text's `start..end` became `text`. */
export interface TextEdit {
  start: number;
  end: number;
  text: string;
}

/**
 * What a change did to the expanded tree, as a processor that keeps a document open gives it: the root's nodes
 * `from..to` became `nodes`, and every node after them moved by `lines` lines and `offset` code points, which their
 * origins don't show.
 */
export interface Splice {
  from: number;
  /** The end of the nodes replaced, in the tree before the change. */
  to: number;
  nodes: ExpandedNode[];
  lines: number;
  offset: number;
  /** The root's inputs after the change. */
  inputs?: Record<string, ExpandedInput>;
  /** The edit that made it; absent when the text didn't change (opening, data loaded again). */
  edit?: TextEdit;
}

/** A tree field as text: a string, number or boolean as written, anything else (absent, null) empty. */
export function textOf(value: unknown): string {
  return typeof value === "string" ? value : typeof value === "number" || typeof value === "boolean" ? String(value) : "";
}

const COMPONENT_TYPES = new Set(["componentContainer", "componentLeaf", "componentInline"]);

export function isComponent(node: ExpandedNode): node is ExpandedComponent {
  return COMPONENT_TYPES.has(node.type);
}

/** A node's blocks in source order: its children, then each slot's, every template instance's included. */
export function contentOf(node: ExpandedNode): ExpandedNode[] {
  const own = node as { children?: ExpandedNode[]; instances?: ExpandedInstance[]; slots?: ExpandedSlot[] };
  const slot = (content: { children?: ExpandedNode[]; instances?: ExpandedInstance[] }) => [
    ...(content.children ?? []),
    ...(content.instances ?? []).flatMap((instance) => instance.children),
  ];
  return [...slot(own), ...(own.slots ?? []).flatMap(slot)];
}

/** Calls `visit` for every node of `nodes`, depth first, a leaf's fallback included. */
export function walk(nodes: readonly ExpandedNode[], visit: (node: ExpandedNode) => void): void {
  for (const node of nodes) {
    visit(node);
    walk(contentOf(node), visit);
    const fallback = (node as { fallback?: ExpandedNode[] }).fallback;
    if (fallback) walk(fallback, visit);
  }
}
