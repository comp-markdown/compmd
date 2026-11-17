// A page frame's images (frame mode): the frame has no network, so the host's page fetches each image of the trees it
// sent the frame, and gives it as a Blob, which the frame shows from a `blob:` URL. Its policy stays as it is, so code in
// the frame still loads nothing itself; and since that code could ask too, the host's page gives only those images.
// An image of one of the host's own schemes (spec §15) is the host's to load: its file comes from the host, in both modes.

import { imageSources, urlScheme, type ExpandedRoot, type ImageSource, type Splice } from "@compmd/render";

/** The host's own URL schemes, checked, and the file of an image of one of them (`ownImages`), when it gives them. */
export interface OwnImages {
  schemes: readonly string[];
  file?: ((src: string) => Promise<Blob | undefined>) | undefined;
}

/** Whether `src` uses one of the host's own `schemes`. */
const isOwn = (src: string, schemes: readonly string[]): boolean => {
  const scheme = urlScheme(src);
  return scheme !== undefined && schemes.includes(scheme);
};

/**
 * The images of the trees and splices the host's page sent its page frame, and their files.
 *
 * @example
 * const images = new PageImages(fetch, document.baseURI, { schemes: ["app"], file: (src) => attachment(src) });
 * images.rendered(tree);
 * await images.file("./logo.png"); // a Blob, or undefined
 */
export class PageImages {
  readonly #fetch: typeof fetch;
  readonly #base: string;
  readonly #own: OwnImages;
  /** The images' `src`, as the frame asks for them. */
  #sources = new Set<string>();
  /** Splices as a processor's JSON text, read only once the frame asks for an image they may have. */
  #unread: string[] = [];

  /**
   * Fetches with `fetch`, a relative `src` against `base`, as an image in the host's page would load; an image of one of
   * the host's own schemes comes from `own.file`, never fetched.
   */
  constructor(fetcher: typeof fetch, base: string, own: OwnImages = { schemes: [] }) {
    // Called on its own: the page's `fetch` throws when called on another object.
    this.#fetch = (input, init) => fetcher(input, init);
    this.#base = base;
    this.#own = own;
  }

  /** The frame renders `tree`: its images are the frame's now. */
  rendered(tree: ExpandedRoot): void {
    this.#sources = new Set(imageSources(tree.children, this.#own.schemes));
    this.#unread = [];
  }

  /** The frame applies `splices`: their images are the frame's too. */
  spliced(splices: readonly (Splice | string)[]): void {
    for (const splice of splices) {
      if (typeof splice === "string") this.#unread.push(splice);
      else this.#add(splice);
    }
  }

  /**
   * The file of the image with `src`, or `undefined` when it isn't one of the frame's, can't be had, or isn't an image.
   * One of the host's own schemes comes from the host; any other is fetched without credentials and without a `Referer`.
   */
  async file(src: string): Promise<Blob | undefined> {
    if (!this.#has(src)) return undefined;
    try {
      const file = isOwn(src, this.#own.schemes) ? await this.#own.file?.(src) : await this.#fetched(src);
      return file?.type.startsWith("image/") ? file : undefined;
    } catch {
      return undefined;
    }
  }

  async #fetched(src: string): Promise<Blob | undefined> {
    const response = await this.#fetch(new URL(src, this.#base), { credentials: "omit", referrerPolicy: "no-referrer" });
    return response.ok ? response.blob() : undefined;
  }

  #has(src: string): boolean {
    if (this.#sources.has(src)) return true;
    for (const splice of this.#unread) this.#add(JSON.parse(splice) as Splice);
    this.#unread = [];
    return this.#sources.has(src);
  }

  #add(splice: Splice): void {
    for (const src of imageSources(splice.nodes, this.#own.schemes)) this.#sources.add(src);
  }
}

/**
 * Host mode's images: one of the host's own schemes from a `blob:` URL of the file `file` gives, asked for once while
 * it has one; any other from its `src`, as the host's page loads it.
 *
 * @example
 * createPage(container, { ownSchemes: ["app"], images: hostImages(["app"], (src) => attachment(src)) });
 */
export function hostImages(schemes: readonly string[], file: (src: string) => Promise<Blob | undefined>): ImageSource {
  const shown = new Map<string, Promise<string | undefined>>();
  return (src) => {
    if (!isOwn(src, schemes)) return Promise.resolve(src);
    let url = shown.get(src);
    if (!url) {
      url = file(src).then(
        (blob) => {
          if (blob) return URL.createObjectURL(blob);
          shown.delete(src);
          return undefined;
        },
        () => {
          shown.delete(src);
          return undefined;
        },
      );
      shown.set(src, url);
    }
    return url;
  };
}
