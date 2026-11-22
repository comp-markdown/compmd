// The files of the fonts of the trees and splices a parent sent its frame (spec §9.4), as the frame asks for them: the
// parent gives a frame only those. A root's fonts are those it uses now, so a font a later tree dropped is still one a
// frame may ask for while it shows the change.

import { fontSources, type ExpandedFont, type Splice } from "@compmd/render";

/**
 * The `src` of each font file sent, a pack's by the URL `packFile` gives its place.
 *
 * @example
 * const fonts = new SentFonts((place) => `/packs/${place}`);
 * fonts.rendered(tree.fonts ?? []);
 * fonts.has("./fonts/serif.woff2"); // true
 */
export class SentFonts {
  readonly #packFile: ((place: string) => string | undefined) | undefined;
  #sources = new Set<string>();
  /** Splices as a processor's JSON text, read only once the frame asks for a file. */
  #unread: string[] = [];

  constructor(packFile?: (place: string) => string | undefined) {
    this.#packFile = packFile;
  }

  /** The frame renders a tree with `fonts`, in place of all it had. */
  rendered(fonts: readonly ExpandedFont[]): void {
    this.#sources = new Set(fontSources(fonts, this.#packFile));
    this.#unread = [];
  }

  /** The frame applies `splices`, beside what it had. */
  spliced(splices: readonly (Splice | string)[]): void {
    for (const splice of splices) {
      if (typeof splice === "string") this.#unread.push(splice);
      else this.#add(splice.fonts ?? []);
    }
  }

  /** Whether `src` is a file of a font sent. */
  has(src: string): boolean {
    if (this.#sources.has(src)) return true;
    for (const splice of this.#unread) this.#add((JSON.parse(splice) as Splice).fonts ?? []);
    this.#unread = [];
    return this.#sources.has(src);
  }

  #add(fonts: readonly ExpandedFont[]): void {
    for (const src of fontSources(fonts, this.#packFile)) this.#sources.add(src);
  }
}
