// Layout and common params as CSS (spec §9.2–§9.4), and the stylesheet every layer of one document writes to.
//
// Every param of a block follows its parent's width (§9.3), so a block's breakpoint params are container queries,
// answered by the nearest box around it that is a container: a box becomes one when blocks placed in it use
// breakpoints, and the page always is.
import type { ExpandedParam } from "./tree.ts";

/** The breakpoints and the widths they start at (spec §9.3), narrowest first. */
export const BREAKPOINTS: readonly (readonly [string, number])[] = [
  ["sm", 480],
  ["md", 768],
  ["lg", 1024],
];

/** `box`'s own layout params (spec §9). */
export const LAYOUT = ["display", "flex-direction", "gap", "align-items", "justify-content", "flex-wrap", "grid-template-columns", "grid-template-rows", "grid-template-areas"];

/** The common params (spec §9.2), and the attributes any component takes. */
export const COMMON = [
  "width", "height", "min-width", "max-width",
  "padding", "padding-inline", "padding-block", "padding-inline-start", "padding-inline-end", "padding-block-start", "padding-block-end",
  "border", "border-radius", "background", "grid-area", "order", "align-self", "justify-self", "aspect-ratio",
  "inset-inline-start", "inset-inline-end", "inset-block-start", "inset-block-end",
  "scroll", "fill", "sticky", "show", "hide",
];
export const ATTRIBUTES = ["class", "id", "lang", "dir"];
const INSETS = ["inset-inline-start", "inset-inline-end", "inset-block-start", "inset-block-end"];

/** Params the renderer applies to a component's outer box rather than passing them on (spec §7.4). */
export const BOX_PARAMS: ReadonlySet<string> = new Set([...COMMON, ...ATTRIBUTES]);

const SPACE: Record<string, string> = {
  none: "0",
  s: "var(--compmd-space-s, 0.5rem)",
  m: "var(--compmd-space-m, 1rem)",
  l: "var(--compmd-space-l, 2rem)",
};
const SURFACE: Record<string, string> = {
  none: "transparent",
  subtle: "var(--compmd-surface-subtle, color-mix(in srgb, currentColor 6%, transparent))",
  strong: "var(--compmd-surface-strong, color-mix(in srgb, currentColor 12%, transparent))",
  accent: "var(--compmd-surface-accent, color-mix(in srgb, var(--compmd-accent, #0969da) 12%, transparent))",
};
const BORDER: Record<string, string> = {
  none: "none",
  subtle: "var(--compmd-border-subtle, 1px solid color-mix(in srgb, currentColor 20%, transparent))",
  strong: "var(--compmd-border-strong, 1px solid color-mix(in srgb, currentColor 45%, transparent))",
};
const RADIUS: Record<string, string> = {
  none: "0",
  s: "var(--compmd-radius-s, 4px)",
  m: "var(--compmd-radius-m, 8px)",
  l: "var(--compmd-radius-l, 16px)",
  full: "var(--compmd-radius-full, 9999px)",
};
const CONTENT_ALIGNMENT: Record<string, string> = { between: "space-between", around: "space-around" };
const ORDER: Record<string, string> = { first: "-1", last: "1", none: "0" };

/** CSS declarations, property to value. */
export type Declarations = Record<string, string>;

/** How the parent box lays out its children, which `fill` and `justify-self` depend on (spec §9.2). */
export type ParentLayout = "block" | "row" | "column" | "grid";

/** A block's CSS: base declarations, those that apply from each breakpoint up, and where it shows or hides. */
export interface BlockCss {
  base: Declarations;
  at: Record<string, Declarations>;
  show?: string;
  hide?: string;
}

const text = (value: unknown): string => (typeof value === "string" ? value : String(value));
const spaces = (value: string) => value.split(/\s+/).map((word) => SPACE[word] ?? word).join(" ");

/** A value is copied into CSS only if it can't end the declaration or the rule it is in. */
const safe = (value: string): boolean => !/[;{}<>\\]/.test(value);

/** The declarations one param gives. */
function declarationsOf(key: string, value: unknown, parent: ParentLayout): Declarations {
  const written = text(value);
  switch (key) {
    case "gap":
    case "padding":
    case "padding-inline":
    case "padding-block":
    case "padding-inline-start":
    case "padding-inline-end":
    case "padding-block-start":
    case "padding-block-end":
      return { [key]: spaces(written) };
    case "background":
      return { background: SURFACE[written] ?? written };
    case "border":
      return { border: BORDER[written] ?? written };
    case "border-radius":
      return { "border-radius": RADIUS[written] ?? written };
    case "justify-content":
      return { "justify-content": CONTENT_ALIGNMENT[written] ?? written };
    case "order":
      return { order: ORDER[written] ?? written };
    case "aspect-ratio":
      return { "aspect-ratio": written === "none" ? "auto" : written };
    case "scroll":
      return value === true ? { overflow: "auto" } : {};
    case "fill":
      if (value !== true) return {};
      return parent === "row" || parent === "grid" ? { "align-self": "stretch" } : { "flex-grow": "1", "min-block-size": "0" };
    case "sticky":
      return written === "none" ? {} : { position: "sticky", [written === "top" ? "inset-block-start" : "inset-block-end"]: "0" };
    case "justify-self":
      if (parent === "grid" || written === "start") return { "justify-self": written };
      // In a flex parent, `end` pushes the block and those after it to the end (spec §9.2).
      return written === "end" ? { "margin-inline-start": "auto" } : { "margin-inline": "auto" };
    default:
      return { [key]: written };
  }
}

/**
 * The CSS of a block's common params, and of its layout params when it is a `box` (`layout`), without the attributes
 * (`class`, `id`, `lang`, `dir`); any other param is the component's own. A block with a position (an inset that
 * isn't `auto`) is taken out of the flow, at each breakpoint where it has one (spec §9.2).
 *
 * @example
 * blockCss([{ key: "gap", breakpoint: null, value: "m" }, { key: "gap", breakpoint: "md", value: "l" }], "block", true);
 * // { base: { gap: "var(--compmd-space-m, 1rem)" }, at: { md: { gap: "var(--compmd-space-l, 2rem)" } } }
 */
export function blockCss(params: readonly ExpandedParam[], parent: ParentLayout, layout = false): BlockCss {
  const css: BlockCss = { base: {}, at: {} };
  for (const { key, breakpoint, value } of params) {
    if (!(COMMON.includes(key) || (layout && LAYOUT.includes(key))) || value === null || value === undefined) continue;
    if (key === "show" || key === "hide") {
      css[key] = text(value);
      continue;
    }
    const declarations = Object.entries(declarationsOf(key, value, parent)).filter(([, v]) => safe(v));
    Object.assign(breakpoint === null ? css.base : (css.at[breakpoint] ??= {}), Object.fromEntries(declarations));
  }
  const insets: Record<string, string> = {};
  for (const level of [null, ...BREAKPOINTS.map(([name]) => name)]) {
    const given = params.filter((p) => INSETS.includes(p.key) && p.breakpoint === level);
    if (given.length === 0) continue;
    for (const { key, value } of given) insets[key] = text(value);
    const placed = Object.values(insets).some((value) => value !== "auto");
    Object.assign(level === null ? css.base : (css.at[level] ??= {}), { position: placed ? "absolute" : "relative" });
  }
  return css;
}

/** How a box with these params lays out its children. */
export function layoutOf(params: readonly ExpandedParam[]): ParentLayout {
  const base = (key: string) => params.find((p) => p.key === key && p.breakpoint === null)?.value;
  const display = base("display");
  if (display === "grid") return "grid";
  if (display === "flex") return base("flex-direction") === "column" ? "column" : "row";
  return "block";
}

/** Whether a block's params use breakpoints, so the box it is placed in must be a container (spec §9.3). */
export function usesBreakpoints(params: readonly ExpandedParam[]): boolean {
  return params.some((p) => p.breakpoint !== null || p.key === "show" || p.key === "hide");
}

/** Whether a block takes the height its parent leaves. */
export function fills(params: readonly ExpandedParam[]): boolean {
  return params.some((p) => p.key === "fill" && p.value === true);
}

/** The value of an attribute param; `class` gives every class, space-separated (spec §12.3 merges them into one). */
export function attributeOf(params: readonly ExpandedParam[], key: string): string | undefined {
  const value = params.find((param) => param.key === key)?.value;
  if (value === undefined || value === null) return undefined;
  return Array.isArray(value) ? value.map(text).join(" ") : text(value);
}

/** The params a host-language component receives as its own: the rest belong to its outer box (spec §7.4). */
export function ownParams(params: readonly ExpandedParam[]): Record<string, unknown> {
  return Object.fromEntries(params.filter((p) => !BOX_PARAMS.has(p.key) && p.breakpoint === null).map((p) => [p.key, p.value]));
}

/** Rules every document needs once: the boxes the renderer makes, and the containers breakpoints query. */
const BASE_RULES = `
.compmd-page { container-type: inline-size; position: relative; }
.compmd-box, .compmd-item, .compmd-component { position: relative; box-sizing: border-box; min-inline-size: 0; }
.compmd-container { container-type: inline-size; }
.compmd-fills { display: flex; flex-direction: column; }
.compmd-alert { border-inline-start: 0.25em solid var(--compmd-border-strong, color-mix(in srgb, currentColor 45%, transparent)); padding-inline-start: 1em; }
.compmd-footnotes { font-size: smaller; }
.compmd-task { list-style: none; }
.compmd-align-left { text-align: start; }
.compmd-align-center { text-align: center; }
.compmd-align-right { text-align: end; }
`;

const declarationsText = (declarations: Declarations) => Object.entries(declarations).map(([property, value]) => `${property}: ${value};`).join(" ");

/**
 * The stylesheet of one document, which every layer writes to: each styled block gets a class and its rules, and
 * `flush` writes them all at once. It is a constructed stylesheet, which a sandbox's CSP allows, or a `<style>`
 * element where constructed stylesheets aren't supported.
 *
 * @example
 * const sheet = new Stylesheet(document);
 * element.classList.add(sheet.style(element, blockCss(params, "block")));
 * sheet.flush();
 */
export class Stylesheet {
  readonly #document: Document;
  readonly #sheet: CSSStyleSheet | undefined;
  readonly #element: HTMLStyleElement | undefined;
  readonly #blocks = new Map<string, BlockCss>();
  readonly #classes = new WeakMap<Element, string>();
  #next = 0;
  #text = "";

  constructor(document: Document) {
    this.#document = document;
    const Sheet = document.defaultView?.CSSStyleSheet;
    if (Sheet && typeof Sheet.prototype.replaceSync === "function" && "adoptedStyleSheets" in document) {
      this.#sheet = new Sheet();
      document.adoptedStyleSheets = [...document.adoptedStyleSheets, this.#sheet];
    } else {
      this.#element = document.head.appendChild(document.createElement("style"));
    }
  }

  /** Sets `element`'s block rules, and returns the class they select it by. */
  style(element: Element, css: BlockCss): string {
    let name = this.#classes.get(element);
    if (name === undefined) {
      name = `compmd-b${String(this.#next++)}`;
      this.#classes.set(element, name);
    }
    this.#blocks.set(name, css);
    return name;
  }

  /** Forgets `element`'s rules, when it is removed. */
  forget(element: Element): void {
    const name = this.#classes.get(element);
    if (name !== undefined) this.#blocks.delete(name);
  }

  /** Writes every rule: the base rules, then each block's, breakpoints in order so wider ones win. */
  flush(): void {
    const base: string[] = [];
    const at = new Map<string, string[]>(BREAKPOINTS.map(([name]) => [name, []]));
    const visibility: string[] = [];
    for (const [name, css] of this.#blocks) {
      if (Object.keys(css.base).length > 0) base.push(`.${name} { ${declarationsText(css.base)} }`);
      for (const [breakpoint, declarations] of Object.entries(css.at)) at.get(breakpoint)?.push(`.${name} { ${declarationsText(declarations)} }`);
      const width = (breakpoint: string | undefined) => BREAKPOINTS.find(([n]) => n === breakpoint)?.[1];
      const shownFrom = width(css.show);
      const hiddenFrom = width(css.hide);
      if (shownFrom !== undefined) visibility.push(`@container (max-width: ${String(shownFrom - 0.02)}px) { .${name} { display: none; } }`);
      if (hiddenFrom !== undefined) visibility.push(`@container (min-width: ${String(hiddenFrom)}px) { .${name} { display: none; } }`);
    }
    const queries = BREAKPOINTS.flatMap(([name, width]) => {
      const rules = at.get(name) ?? [];
      return rules.length === 0 ? [] : [`@container (min-width: ${String(width)}px) {\n${rules.join("\n")}\n}`];
    });
    const next = [BASE_RULES, ...base, ...queries, ...visibility].join("\n");
    if (next === this.#text) return;
    this.#text = next;
    if (this.#sheet) this.#sheet.replaceSync(next);
    else if (this.#element) this.#element.textContent = next;
  }

  /** The rules as written by the last `flush`. */
  get text(): string {
    return this.#text;
  }

  /** Removes the stylesheet from its document. */
  remove(): void {
    if (this.#sheet) this.#document.adoptedStyleSheets = this.#document.adoptedStyleSheets.filter((sheet) => sheet !== this.#sheet);
    this.#element?.remove();
  }
}
