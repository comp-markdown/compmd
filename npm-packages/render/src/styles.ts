// Layout and common params as CSS (spec §9.2–§9.4), and the stylesheet every layer of one document writes to.
//
// Every param of a block follows its parent's width (§9.3), so a block's breakpoint params are container queries,
// answered by the nearest box around it that is a container: a box becomes one when blocks placed in it use
// breakpoints, and the page always is.
import { CODE_FRAME_RULES } from "./code-frame.ts";
import { familyCss } from "./fonts.ts";
import { followsDirection, gradientFor } from "./gradient.ts";
import { IMAGE_BOX_RULES } from "./image-box.ts";
import type { ExpandedParam } from "./tree.ts";

/** The breakpoints and the widths they start at (spec §9.3), narrowest first. */
export const BREAKPOINTS: readonly (readonly [string, number])[] = [
  ["sm", 480],
  ["md", 768],
  ["lg", 1024],
];

/** `box`'s own layout params (spec §9). */
export const LAYOUT = ["display", "flex-direction", "gap", "align-items", "justify-content", "flex-wrap", "grid-template-columns", "grid-template-rows", "grid-auto-rows", "grid-template-areas"];

/** The common params (spec §9.2), and the attributes any component takes. */
export const COMMON = [
  "width", "height", "min-width", "max-width",
  "padding", "padding-inline", "padding-block", "padding-inline-start", "padding-inline-end", "padding-block-start", "padding-block-end",
  "border", "border-inline-start", "border-inline-end", "border-block-start", "border-block-end", "border-radius",
  "shadow", "rotate", "background",
  "font-size", "font-weight", "font-style", "font-family", "color", "text-align",
  "line-height", "letter-spacing", "text-transform", "text-decoration",
  "grid-area", "order", "align-self", "justify-self", "aspect-ratio",
  "object-fit", "object-position",
  "inset-inline-start", "inset-inline-end", "inset-block-start", "inset-block-end",
  "scroll", "fill", "sticky", "stretch-link", "show", "hide",
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
const SHADOW: Record<string, string> = {
  none: "none",
  s: "var(--compmd-shadow-s, 0 1px 2px color-mix(in srgb, currentColor 20%, transparent))",
  m: "var(--compmd-shadow-m, 0 2px 6px color-mix(in srgb, currentColor 25%, transparent))",
  l: "var(--compmd-shadow-l, 0 6px 16px color-mix(in srgb, currentColor 30%, transparent))",
};
const FONT_SIZE: Record<string, string> = {
  s: "var(--compmd-text-s, 0.875em)",
  m: "var(--compmd-text-m, 1em)",
  l: "var(--compmd-text-l, 1.25em)",
  xl: "var(--compmd-text-xl, 1.5em)",
};
const FONT_FAMILY: Record<string, string> = {
  body: "var(--compmd-font-body, inherit)",
  heading: "var(--compmd-font-heading, inherit)",
  mono: "var(--compmd-font-mono, ui-monospace, monospace)",
};
const TEXT_COLOR: Record<string, string> = {
  muted: "var(--compmd-text-muted, color-mix(in srgb, currentColor 65%, transparent))",
  accent: "var(--compmd-text-accent, var(--compmd-accent, #0969da))",
};
const LINE_HEIGHT: Record<string, string> = {
  tight: "var(--compmd-leading-tight, 1.2)",
  normal: "normal",
  loose: "var(--compmd-leading-loose, 1.8)",
};
const LETTER_SPACING: Record<string, string> = {
  tight: "var(--compmd-tracking-tight, -0.01em)",
  normal: "normal",
  wide: "var(--compmd-tracking-wide, 0.05em)",
};
const CONTENT_ALIGNMENT: Record<string, string> = { between: "space-between", around: "space-around" };
const ORDER: Record<string, string> = { first: "-1", last: "1", none: "0" };

/** CSS declarations, property to value. */
export type Declarations = Record<string, string>;

/** How the parent box lays out its children, which `fill` and `justify-self` depend on (spec §9.2). */
export type ParentLayout = "block" | "row" | "column" | "grid";

/**
 * A block's CSS: base declarations, those that apply from each breakpoint up, where it shows or hides, and, when its
 * background follows the text's direction (a gradient's logical side or angle), its backgrounds in right-to-left text.
 */
export interface BlockCss {
  base: Declarations;
  at: Record<string, Declarations>;
  show?: string;
  hide?: string;
  rtl?: { base: Declarations; at: Record<string, Declarations> };
}

const text = (value: unknown): string => (typeof value === "string" ? value : String(value));
const spaces = (value: string) => value.split(/\s+/).map((word) => SPACE[word] ?? word).join(" ");

/** Whether `value`'s parentheses close in order, so it can't run on past its own value. */
function balanced(value: string): boolean {
  let depth = 0;
  for (const char of value) {
    if (char === "(") depth += 1;
    else if (char === ")" && --depth < 0) return false;
  }
  return depth === 0;
}

/**
 * A value is copied into CSS only if it can't end the declaration or the rule it is in, reach the network, or hide
 * what follows: no `;`, braces, `<`, `>`, backslash, quote, comment or `url(`, and its parentheses balanced (audit S5;
 * the processor checks the grammar first, and this holds whatever reaches the renderer).
 */
const safe = (value: string): boolean => !/[;{}<>\\"'`]|\/\*|url\s*\(/i.test(value) && balanced(value);

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
      return { background: SURFACE[written] ?? gradientFor(written, "ltr") };
    case "border":
    case "border-inline-start":
    case "border-inline-end":
    case "border-block-start":
    case "border-block-end":
      return { [key]: BORDER[written] ?? written };
    case "shadow":
      return { "box-shadow": SHADOW[written] ?? "none" };
    case "font-size":
      return { "font-size": FONT_SIZE[written] ?? written };
    case "font-family": {
      // A token, or a font of the document's own by its family (`fonts.ts`).
      const family = FONT_FAMILY[written] ?? familyCss(written);
      return family === undefined ? {} : { "font-family": family };
    }
    case "color":
      return { color: TEXT_COLOR[written] ?? written };
    case "line-height":
      return { "line-height": LINE_HEIGHT[written] ?? written };
    case "letter-spacing":
      return { "letter-spacing": LETTER_SPACING[written] ?? written };
    case "rotate":
      return { rotate: written };
    // Not CSS of the box's own: the box and its first link are marked (stretch-link.ts), and the base rules draw it.
    case "stretch-link":
      return {};
    case "border-radius":
      return { "border-radius": RADIUS[written] ?? written };
    case "justify-content":
      return { "justify-content": CONTENT_ALIGNMENT[written] ?? written };
    case "order":
      return { order: ORDER[written] ?? written };
    case "aspect-ratio":
      return { "aspect-ratio": written === "none" ? "auto" : written };
    // What a component draws in its box, which the component gets as `fit` (spec §7.4): nothing for the box itself.
    case "object-fit":
    case "object-position":
      return {};
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
  const rtl = rightToLeftBackgrounds(params);
  if (rtl) css.rtl = rtl;
  return css;
}

/**
 * A block's backgrounds in right-to-left text, when one of them follows the text's direction; then every one of them,
 * at every breakpoint, so a plain color from `md` up still wins over a right-to-left gradient below it.
 */
function rightToLeftBackgrounds(params: readonly ExpandedParam[]): BlockCss["rtl"] {
  const backgrounds = params.filter((p) => p.key === "background" && typeof p.value === "string");
  if (!backgrounds.some((p) => followsDirection(text(p.value)))) return undefined;
  const rtl: NonNullable<BlockCss["rtl"]> = { base: {}, at: {} };
  for (const { breakpoint, value } of backgrounds) {
    const written = text(value);
    const background = SURFACE[written] ?? gradientFor(written, "rtl");
    if (safe(background)) Object.assign(breakpoint === null ? rtl.base : (rtl.at[breakpoint] ??= {}), { background });
  }
  return rtl;
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

/**
 * Rules every document needs once: the boxes the renderer makes (`render=both`'s stacked, which a host's
 * `.compmd-both` rule can set side by side), a code block's frame (`code-frame.ts`), the containers breakpoints query, a code block's lines as blocks (so a
 * host can size one: `.compmd-line { min-height }`), and the look of what Markdown has no CSS for. A box's padding is its own: the margins of its first and last child are trimmed, as
 * `margin-trim: block` would, at the specificity of `:first-child`, so a host's own rules still win.
 *
 * A host-language component's box is a grid of one column (docs/sizing-components.md, *The mechanism*), so a
 * percentage height in it is of its cell however the box gets its height; an inline one is an inline grid. At one
 * class, so the rules that hide a box (`show`, `hide`) still do.
 *
 * On paper (docs/printing.md): the page is light, a component that prints its fallback shows it instead of itself, a
 * component, a table, a code block or an image is kept whole where it fits, and a heading stays with what follows it;
 * all at no specificity, so a host's print styles win.
 */
const BASE_RULES = `
.compmd-page { container-type: inline-size; position: relative; }
.compmd-box, .compmd-item, .compmd-component { position: relative; box-sizing: border-box; min-inline-size: 0; }
.compmd-component { display: grid; grid-template-columns: minmax(0, 1fr); grid-auto-rows: minmax(0, auto); }
:where(span).compmd-component { display: inline-grid; }
:where(span).compmd-box { display: inline-block; }
.compmd-stretched::after { content: ""; position: absolute; inset: 0; }
:where(.compmd-stretch-link) a:not(.compmd-stretched) { position: relative; z-index: 1; }
.compmd-container { container-type: inline-size; }
.compmd-fills { display: flex; flex-direction: column; }
:where(.compmd-both) { display: flex; flex-direction: column; }
:where(.compmd-box, .compmd-item, .compmd-component) > :first-child { margin-block-start: 0; }
:where(.compmd-box, .compmd-item, .compmd-component) > :last-child { margin-block-end: 0; }
.compmd-alert { border-inline-start: 0.25em solid var(--compmd-border-strong, color-mix(in srgb, currentColor 45%, transparent)); padding-inline-start: 1em; }
.compmd-quote { margin-inline: 0; border-inline-start: 0.25em solid var(--compmd-border-subtle-color, color-mix(in srgb, currentColor 25%, transparent)); padding-inline-start: 1em; }
.compmd-footnotes { font-size: smaller; }
.compmd-tight > li > p { margin-block: 0; }
.compmd-task { list-style: none; }
.compmd-task > input:first-child { margin-block: 0; margin-inline: 0 0.5em; }
.compmd-task > input:first-child + p { display: inline; }
.compmd-input { display: inline-flex; align-items: center; gap: 0.5em; }
.compmd-pending { opacity: 0.6; }
.compmd-revealed { outline: 2px solid var(--compmd-accent, #0969da); outline-offset: 2px; }
.compmd-align-left { text-align: start; }
.compmd-align-center { text-align: center; }
.compmd-align-right { text-align: end; }
.compmd-line { display: block; }
.compmd-print-wrap { display: contents; }
.compmd-print-paper { display: none; }
@media print {
  :where(.compmd-page) { color-scheme: light; }
  .compmd-print-wrap > .compmd-component { display: none !important; }
  div.compmd-print-paper { display: block; }
  span.compmd-print-paper { display: inline; }
  :where(.compmd-page) :where(.compmd-component, .compmd-box, table, pre, img, figure) { break-inside: avoid; }
  :where(.compmd-page) :where(h1, h2, h3, h4, h5, h6) { break-after: avoid; }
}
${IMAGE_BOX_RULES}${CODE_FRAME_RULES}`;

const declarationsText = (declarations: Declarations) => Object.entries(declarations).map(([property, value]) => `${property}: ${value};`).join(" ");

/**
 * The one sheet a document has for every {@link Stylesheet} in it: the base rules once, then each one's rules. The
 * document adopts it with the first and drops it with the last, so a page more is no change to the document's sheets,
 * which would restyle all of it. It is a constructed stylesheet, which a sandbox's CSP allows, or a `<style>` element
 * where constructed stylesheets aren't supported.
 */
class DocumentSheet {
  static readonly #all = new WeakMap<Document, DocumentSheet>();
  readonly #document: Document;
  readonly #users = new Set<Stylesheet>();
  #sheet: CSSStyleSheet | undefined;
  #element: HTMLStyleElement | undefined;
  #next = 0;
  #text = "";

  static of(document: Document): DocumentSheet {
    let sheet = DocumentSheet.#all.get(document);
    if (!sheet) DocumentSheet.#all.set(document, (sheet = new DocumentSheet(document)));
    return sheet;
  }

  private constructor(document: Document) {
    this.#document = document;
  }

  /**
   * A class no block of the document has had: never one used before, a destroyed page's included, so a rule can
   * only ever match the block it was written for.
   */
  name(): string {
    return `compmd-b${String(this.#next++)}`;
  }

  join(user: Stylesheet): void {
    this.#users.add(user);
    if (this.#users.size > 1) return;
    this.write();
    const Sheet = this.#document.defaultView?.CSSStyleSheet;
    if (Sheet && typeof Sheet.prototype.replaceSync === "function" && "adoptedStyleSheets" in this.#document) {
      this.#sheet = new Sheet();
      this.#sheet.replaceSync(this.#text);
      this.#document.adoptedStyleSheets = [...this.#document.adoptedStyleSheets, this.#sheet];
    } else {
      this.#element = this.#document.head.appendChild(this.#document.createElement("style"));
      this.#element.textContent = this.#text;
    }
  }

  leave(user: Stylesheet): void {
    if (!this.#users.delete(user)) return;
    if (this.#users.size > 0) {
      this.write();
      return;
    }
    if (this.#sheet) this.#document.adoptedStyleSheets = this.#document.adoptedStyleSheets.filter((sheet) => sheet !== this.#sheet);
    this.#element?.remove();
    this.#sheet = undefined;
    this.#element = undefined;
  }

  /** Writes the sheet when its rules changed: a stylesheet whose blocks have none changes nothing. */
  write(): void {
    const next = [BASE_RULES, ...[...this.#users].map((user) => user.rules).filter(Boolean)].join("\n");
    if (next === this.#text) return;
    this.#text = next;
    if (this.#sheet) this.#sheet.replaceSync(next);
    else if (this.#element) this.#element.textContent = next;
  }
}

/**
 * The rules of one page, or of a region with a stylesheet of its own, which every layer of it writes to: each styled
 * block gets a class and its rules, and `flush` writes them all at once, into the one sheet its document has for all
 * of them. So many pages in one document share the base rules, and a page whose blocks have no rules never changes
 * the document's styles.
 *
 * @example
 * const sheet = new Stylesheet(document);
 * element.classList.add(sheet.style(element, blockCss(params, "block")));
 * sheet.flush();
 */
export class Stylesheet {
  readonly #shared: DocumentSheet;
  readonly #blocks = new Map<string, BlockCss>();
  readonly #classes = new WeakMap<Element, string>();
  #rules = "";
  #text = "";

  constructor(document: Document) {
    this.#shared = DocumentSheet.of(document);
    this.#shared.join(this);
  }

  /** Sets `element`'s block rules, and returns the class they select it by, which is its own in the document. */
  style(element: Element, css: BlockCss): string {
    let name = this.#classes.get(element);
    if (name === undefined) {
      name = this.#shared.name();
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
      if (css.rtl && Object.keys(css.rtl.base).length > 0) base.push(`.${name}:dir(rtl) { ${declarationsText(css.rtl.base)} }`);
      for (const [breakpoint, declarations] of Object.entries(css.at)) at.get(breakpoint)?.push(`.${name} { ${declarationsText(declarations)} }`);
      for (const [breakpoint, declarations] of Object.entries(css.rtl?.at ?? {})) at.get(breakpoint)?.push(`.${name}:dir(rtl) { ${declarationsText(declarations)} }`);
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
    this.#rules = [...base, ...queries, ...visibility].join("\n");
    this.#text = [BASE_RULES, ...base, ...queries, ...visibility].join("\n");
    this.#shared.write();
  }

  /** Its blocks' rules as the last `flush` wrote them, without the base rules. */
  get rules(): string {
    return this.#rules;
  }

  /** The base rules and its own, as written by the last `flush`: all a page of its own needs, as one built ahead. */
  get text(): string {
    return this.#text;
  }

  /** Removes its rules from the document, and the document's sheet with the last stylesheet in it. */
  remove(): void {
    this.#shared.leave(this);
  }
}
