// What every part's drawing uses: making elements, and reading a part's text, attributes and flags.

import type { Part } from "./parse.ts";

/** What a part's drawing is given: its document, and a way to draw its children. */
export interface Drawing {
  document: Document;
  /** Draws a part, any part, as the outline has it. */
  part: (part: Part) => HTMLElement;
}

/** Draws one part. */
export type DrawPart = (part: Part, drawing: Drawing) => HTMLElement;

/** An element of `tag` with the classes `wf-<name>` for each name, holding `children`; strings become text. */
export function element(document: Document, tag: string, names: readonly string[], ...children: (Node | string | undefined)[]): HTMLElement {
  const made = document.createElement(tag);
  made.className = names.map((name) => `wf-${name}`).join(" ");
  for (const child of children) if (child !== undefined) made.append(child);
  return made;
}

/** Its children, drawn. */
export const childrenOf = (part: Part, drawing: Drawing): HTMLElement[] => part.children.map(drawing.part);

/** Its positional text at `index`, or undefined. */
export function textOf(part: Part, index = 0): string | undefined {
  const value = part.positionals[index];
  return value?.kind === "string" ? value.text : undefined;
}

/** Its attribute `name` as text, or undefined. */
export function attribute(part: Part, name: string): string | undefined {
  const value = part.attributes[name];
  if (value === undefined) return undefined;
  if (value.kind === "string" || value.kind === "word") return value.text;
  if (value.kind === "number") return `${String(value.value)}${value.unit}`;
  return `${String(value.from)}-${String(value.to)}`;
}

/** Its attribute `name` as a number, or undefined. */
export function numberOf(part: Part, name: string): number | undefined {
  const value = part.attributes[name];
  return value?.kind === "number" ? value.value : undefined;
}

/** Whether it has the flag `name`. */
export const flag = (part: Part, name: string): boolean => part.flags.includes(name);

/** The `wf-` classes for those of `names` that are its flags. */
export const flagged = (part: Part, names: readonly string[]): string[] => names.filter((name) => flag(part, name));

/** A small badge, when `text` is given. */
export const badge = (document: Document, text: string | undefined): HTMLElement | undefined => (text === undefined ? undefined : element(document, "span", ["badge"], text));

/** An icon: a small square with its name's first letter, as Wireloom draws an icon it doesn't know. */
export const icon = (document: Document, name: string | undefined): HTMLElement | undefined => {
  if (name === undefined) return undefined;
  const drawn = element(document, "span", ["icon"], (name.at(0) ?? "").toUpperCase());
  drawn.title = name;
  return drawn;
};
