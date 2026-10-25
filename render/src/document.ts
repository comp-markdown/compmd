// What only the whole document has: footnote numbers, link reference definitions, and the plain text of a fallback.
import { contentOf, isComponent, textOf, walk, type ExpandedNode } from "./tree.ts";

/** A link reference definition (`[id]: url "title"`), by its normalized identifier. */
export interface LinkDefinition {
  url: string;
  title?: string | null;
}

/** The document's footnotes, as GitHub shows them: references numbered in reading order, definitions in that order. */
export interface Footnotes {
  numbers: ReadonlyMap<string, number>;
  /** Each definition's content, in the order of its number (unreferenced ones last). */
  definitions: readonly { identifier: string; number: number; children: ExpandedNode[] }[];
}

/**
 * The footnotes of `nodes`, the whole document's blocks.
 *
 * @example
 * footnotesOf(tree.children).numbers.get("note"); // 1, for the first footnote referenced
 */
export function footnotesOf(nodes: readonly ExpandedNode[]): Footnotes {
  const numbers = new Map<string, number>();
  const definitions = new Map<string, ExpandedNode[]>();
  walk(nodes, (node) => {
    if (isComponent(node)) return;
    const identifier = node.identifier;
    if (typeof identifier !== "string") return;
    if (node.type === "footnoteReference" && !numbers.has(identifier)) numbers.set(identifier, numbers.size + 1);
    if (node.type === "footnoteDefinition") definitions.set(identifier, node.children ?? []);
  });
  for (const identifier of definitions.keys()) if (!numbers.has(identifier)) numbers.set(identifier, numbers.size + 1);
  const ordered = [...definitions].map(([identifier, children]) => ({ identifier, number: numbers.get(identifier) ?? 0, children }));
  return { numbers, definitions: ordered.sort((a, b) => a.number - b.number) };
}

/** The link reference definitions of `nodes`, the whole document's blocks. */
export function definitionsOf(nodes: readonly ExpandedNode[]): Map<string, LinkDefinition> {
  const definitions = new Map<string, LinkDefinition>();
  walk(nodes, (node) => {
    if (node.type !== "definition" || typeof node.identifier !== "string" || definitions.has(node.identifier)) return;
    definitions.set(node.identifier, { url: textOf(node.url), title: (node.title as string | null | undefined) ?? null });
  });
  return definitions;
}

/** The plain text of `nodes`, as a fallback's `text` (spec §7.4). */
export function plainText(nodes: readonly ExpandedNode[]): string {
  return nodes
    .map((node) => {
      if (isComponent(node)) return plainText(contentOf(node));
      if (typeof node.value === "string" && node.type !== "html") return node.value;
      if (node.type === "wikiLink") return textOf(node.label ?? node.target);
      return plainText(contentOf(node));
    })
    .join("");
}
