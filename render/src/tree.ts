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
