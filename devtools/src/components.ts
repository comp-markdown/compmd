// The Components tab's lists, from what the processor gives, and how it writes a component's kind, params, slots and
// source.

import type { Components, Declaration, DeclaredType, ListedComponent, Range } from "@compmd/processor";

/** A component with where it comes from: the file that defines it, `undefined` for the document itself. */
export interface Sourced extends ListedComponent {
  source: string | undefined;
}

/** A name the document uses: a component, or `unknown` when it stands for none. */
export type UsedName = Sourced | { name: string; unknown: true; uses: Required<Range>[] };

/** The tab's three lists: what the document defines, what it uses, and what it could use but doesn't. */
export interface ComponentLists {
  /** Its own definitions, used or not. */
  defined: ListedComponent[];
  /** Every name it invokes, wherever it comes from, by name. */
  used: UsedName[];
  /** The imported and built-in components it doesn't use, by name. */
  available: Sourced[];
}

/** The tab's lists of `components`. */
export function listsOf(components: Components): ComponentLists {
  const all = components.groups.flatMap((group) => group.components.map((component) => ({ ...component, source: group.file })));
  const byName = (one: { name: string }, other: { name: string }) => one.name.localeCompare(other.name);
  const unknown = components.unknown.map((name) => ({ ...name, unknown: true as const }));
  return {
    defined: all.filter((component) => component.source === undefined),
    used: [...all.filter((component) => component.uses.length > 0), ...unknown].sort(byName),
    available: all.filter((component) => component.source !== undefined && component.uses.length === 0).sort(byName),
  };
}

/** Where a component comes from, as the tab shows it: `this document`, a file, a pack without its `index.comp.md`. */
export function sourceText(source: string | undefined): string {
  return source === undefined ? "this document" : source.replace(/\/index\.comp\.md$/u, "");
}

const KINDS: Record<string, string> = {
  compmd: "CompMD",
  processor: "Built in",
  jsx: "JSX",
  tsx: "TSX",
  svelte: "Svelte",
  js: "JavaScript",
  ts: "TypeScript",
};

/** What implements a component, as a reader names it: `CompMD`, `JSX`, `Built in`; another language as written. */
export function kindLabel(kind: string): string {
  return KINDS[kind] ?? kind;
}

/** A declared type as a signature writes it: `number`, `draft | final`, `input<date>`. */
export function typeText(type: DeclaredType): string {
  if (typeof type === "string") return type;
  if ("enum" in type) return type.enum.join(" | ");
  return `input<${typeText(type.input)}>`;
}

/** A param as a signature writes it: `tone: string?`. */
export function paramText({ name, type, optional }: Declaration): string {
  return `${name}: ${typeText(type)}${optional ? "?" : ""}`;
}

/** A component's slots in a few words, `default, items (each rows)`; `undefined` when it takes no body. */
export function slotsText({ signature }: ListedComponent): string | undefined {
  const slots = signature?.slots;
  if (!slots) return undefined;
  return slots.map((slot) => (typeof slot === "string" ? slot : `${slot.name} (each ${slot.each})`)).join(", ");
}

/** How often the document uses a component. */
export function usesText(count: number): string {
  if (count === 0) return "not used";
  return count === 1 ? "1 use" : `${String(count)} uses`;
}
