// Fonts of a document's own (spec §9.4, §12.3): each font the expanded tree's root lists becomes `@font-face` rules,
// one per file, with `font-display: swap`, so its fallback shows until it loads, and stays where it can't. A box's
// `font-family` names the font through a custom property (`styles.ts`), set on the element the document renders in,
// which holds the font and its fallback's: so a page and an example in it, each with a font of the same name, keep
// their own.
import type { ExpandedFont, ExpandedFontFile } from "./tree.ts";

/**
 * Where a font's files come from: the URL to load a file's `src` from (a pack's by the URL `packFile` gave it), or
 * `undefined` when it can't be had, which shows the fallback. Without it, each loads from its `src`.
 */
export type FontSource = (src: string) => Promise<string | undefined>;

/** The CSS of the type tokens a font falls back to, as the `font-family` tokens are (spec §9.4). */
const FALLBACK: Record<ExpandedFont["fallback"], string> = {
  body: "var(--compmd-font-body, system-ui, sans-serif)",
  heading: "var(--compmd-font-heading, system-ui, sans-serif)",
  mono: "var(--compmd-font-mono, ui-monospace, monospace)",
};

/** A font's family, as a box's `font-family` names it: only a name's characters, so it can't end a rule. */
const isFamily = (family: string): boolean => /^[A-Za-z0-9_-]+$/.test(family);

/** The custom property that holds the font `family` and its fallback. */
export const familyProperty = (family: string): string => `--compmd-family-${family}`;

/** The CSS of a box's `font-family` naming the font `family`; `undefined` for one that can't be a family. */
export function familyCss(family: string): string | undefined {
  return isFamily(family) ? `var(${familyProperty(family)})` : undefined;
}

/** The format a file's URL says. */
const formatOf = (url: string): string => (url.toLowerCase().split(/[?#]/)[0]?.endsWith(".woff") ? "woff" : "woff2");

/** A URL as CSS's `url()` takes it, quoted. */
const urlCss = (url: string): string => `url("${url.replace(/["\\\n]/g, (c) => `\\${c.charCodeAt(0).toString(16)} `)}")`;

/**
 * The `src` of every file of `fonts`, as the renderer asks a {@link FontSource} for it: a pack's file by the URL
 * `packFile` gives its place, none without it.
 *
 * @example
 * fontSources(tree.fonts ?? [], (place) => `/packs/${place}`); // ["./fonts/serif.woff2", "/packs/@acme/look@1.0.0/fonts/sans.woff2"]
 */
export function fontSources(fonts: readonly ExpandedFont[], packFile?: (place: string) => string | undefined): string[] {
  return fonts.flatMap((font) => font.files.flatMap((file) => (file.pack === true ? packFile?.(file.url) : file.url) ?? []));
}

/** A file of a face a page declared: the face's name, the URL it loads from, its format, weight and style. */
export interface FontFaceFile {
  name: string;
  url: string;
  format: string;
  weight: ExpandedFontFile["weight"];
  style: ExpandedFontFile["style"];
}

/** The `@font-face` rule of a face's file. */
const ruleOf = ({ name, url, format, weight, style }: FontFaceFile): string =>
  `@font-face { font-family: "${name}"; src: ${urlCss(url)} format("${format}"); font-weight: ${weight === "bold" ? "700" : "400"}; font-style: ${style}; font-display: swap; }`;

/** The faces the pages of this window declared, by name, with their files. */
const DECLARED = new Map<string, FontFaceFile[]>();

/**
 * The files of the faces of the document's own fonts that `family`, a computed `font-family`, names: what a frame that
 * draws in it needs, which the page's `@font-face` rules don't reach (spec §9.4). None for a family of tokens alone.
 *
 * @example
 * fontFacesIn(getComputedStyle(box).fontFamily); // [{ name: "compmd-font-0-serif", url: "/fonts/serif.woff2", … }]
 */
export function fontFacesIn(family: string): FontFaceFile[] {
  return family.split(",").flatMap((part) => DECLARED.get(part.trim().replace(/^["']|["']$/g, "")) ?? []);
}

/** Every set of fonts so far, which names each one's faces apart from the others'. */
let made = 0;

/**
 * The fonts of what renders in `element`, in a stylesheet of their own on its document, written again whenever the
 * tree's fonts change.
 *
 * @example
 * const fonts = new FontFaces(root, (place) => `/packs/${place}`);
 * await fonts.set(tree.fonts ?? []);
 */
export class FontFaces {
  readonly #element: HTMLElement;
  readonly #document: Document;
  readonly #packFile: ((place: string) => string | undefined) | undefined;
  readonly #source: FontSource | undefined;
  /** What names this set's faces apart: `compmd-font-<n>-<family>`. */
  readonly #prefix = `compmd-font-${String(made++)}`;
  #sheet: CSSStyleSheet | HTMLStyleElement | undefined;
  /** The custom properties set on the element. */
  #properties: string[] = [];
  /** What the fonts were last set to, so the same fonts aren't loaded again. */
  #written = "";
  /** Each file's URL, by its `src`, as the source gave it. */
  readonly #urls = new Map<string, Promise<string | undefined>>();
  /** The names of the faces this set declared, in `DECLARED`. */
  #declared: string[] = [];

  constructor(element: HTMLElement, packFile?: (place: string) => string | undefined, source?: FontSource) {
    this.#element = element;
    this.#document = element.ownerDocument;
    this.#packFile = packFile;
    this.#source = source;
  }

  /** Writes the rules of `fonts`, once their files' URLs are known; resolves when they are written. */
  async set(fonts: readonly ExpandedFont[]): Promise<void> {
    const key = JSON.stringify(fonts);
    if (key === this.#written) return;
    this.#written = key;
    const named = fonts.filter((font) => isFamily(font.family));
    const faces = await Promise.all(named.map((font) => this.#faces(font)));
    // A newer `set` overtook this one: its rules win.
    if (key !== this.#written) return;
    this.#write(faces.flat().map(ruleOf).join("\n"));
    this.#forget();
    for (const [index, font] of named.entries()) {
      DECLARED.set(this.#name(font), faces[index] ?? []);
      this.#declared.push(this.#name(font));
    }
    for (const name of this.#properties) this.#element.style.removeProperty(name);
    this.#properties = named.map((font) => familyProperty(font.family));
    for (const font of named) this.#element.style.setProperty(familyProperty(font.family), `"${this.#name(font)}", ${FALLBACK[font.fallback]}`);
  }

  /** Resolves once the fonts the page uses have loaded, or failed to: what a print waits for. */
  loaded(): Promise<void> {
    // A document without the font loading API (a DOM for tests) has no fonts to wait for.
    const fonts = (this.#document as Partial<Document>).fonts;
    return fonts ? fonts.ready.then(() => undefined) : Promise.resolve();
  }

  /** Takes the fonts' rules and properties away. */
  remove(): void {
    const sheet = this.#sheet;
    if (sheet instanceof HTMLStyleElement) sheet.remove();
    else if (sheet) this.#document.adoptedStyleSheets = this.#document.adoptedStyleSheets.filter((known) => known !== sheet);
    for (const name of this.#properties) this.#element.style.removeProperty(name);
    this.#forget();
    this.#sheet = undefined;
    this.#properties = [];
    this.#written = "";
  }

  #name(font: ExpandedFont): string {
    return `${this.#prefix}-${font.family}`;
  }

  /** Takes this set's faces out of those {@link fontFacesIn} finds. */
  #forget(): void {
    for (const name of this.#declared) DECLARED.delete(name);
    this.#declared = [];
  }

  /** The faces of `font`, one per file that can be had. */
  async #faces(font: ExpandedFont): Promise<FontFaceFile[]> {
    const faces = await Promise.all(
      font.files.map(async (file): Promise<FontFaceFile | undefined> => {
        const url = await this.#url(file);
        return url === undefined ? undefined : { name: this.#name(font), url, format: formatOf(file.url), weight: file.weight, style: file.style };
      }),
    );
    return faces.filter((face) => face !== undefined);
  }

  /** The URL a file loads from: a pack's by `packFile`, then through the source, when there is one. */
  #url(file: ExpandedFontFile): Promise<string | undefined> {
    const src = file.pack === true ? this.#packFile?.(file.url) : file.url;
    if (src === undefined) return Promise.resolve(undefined);
    if (!this.#source) return Promise.resolve(src);
    let url = this.#urls.get(src);
    if (!url) {
      url = this.#source(src).catch(() => undefined);
      this.#urls.set(src, url);
    }
    return url;
  }

  #write(css: string): void {
    // A page with no fonts of its own adds no stylesheet to the document.
    if (!this.#sheet && css === "") return;
    if (!this.#sheet) {
      const Sheet = this.#document.defaultView?.CSSStyleSheet;
      if (Sheet && "adoptedStyleSheets" in this.#document) {
        this.#sheet = new Sheet();
        this.#document.adoptedStyleSheets = [...this.#document.adoptedStyleSheets, this.#sheet];
      } else {
        this.#sheet = this.#document.head.appendChild(this.#document.createElement("style"));
      }
    }
    if (this.#sheet instanceof HTMLStyleElement) this.#sheet.textContent = css;
    else this.#sheet.replaceSync(css);
  }
}
