// A page frame's images (frame mode): the frame has no network, so the host's page fetches each image of the trees it
// sent the frame, and gives it as a Blob, which the frame shows from a `blob:` URL. Its policy stays as it is, so code in
// the frame still loads nothing itself; and since that code could ask too, the host's page gives only those images.

import { imageSources, type ExpandedRoot, type Splice } from "@compmd/render";

/**
 * The images of the trees and splices the host's page sent its page frame, and their files.
 *
 * @example
 * const images = new PageImages(fetch, document.baseURI);
 * images.rendered(tree);
 * await images.file("./logo.png"); // a Blob, or undefined
 */
export class PageImages {
  readonly #fetch: typeof fetch;
  readonly #base: string;
  /** The images' `src`, as the frame asks for them. */
  #sources = new Set<string>();
  /** Splices as a processor's JSON text, read only once the frame asks for an image they may have. */
  #unread: string[] = [];

  /** Fetches with `fetch`, a relative `src` against `base`, as an image in the host's page would load. */
  constructor(fetcher: typeof fetch, base: string) {
    // Called on its own: the page's `fetch` throws when called on another object.
    this.#fetch = (input, init) => fetcher(input, init);
    this.#base = base;
  }

  /** The frame renders `tree`: its images are the frame's now. */
  rendered(tree: ExpandedRoot): void {
    this.#sources = new Set(imageSources(tree.children));
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
   * The file of the image with `src`, or `undefined` when it isn't one of the frame's, can't be fetched, or isn't an
   * image. It is fetched without credentials and without a `Referer`.
   */
  async file(src: string): Promise<Blob | undefined> {
    if (!this.#has(src)) return undefined;
    try {
      const response = await this.#fetch(new URL(src, this.#base), { credentials: "omit", referrerPolicy: "no-referrer" });
      if (!response.ok) return undefined;
      const file = await response.blob();
      return file.type.startsWith("image/") ? file : undefined;
    } catch {
      return undefined;
    }
  }

  #has(src: string): boolean {
    if (this.#sources.has(src)) return true;
    for (const splice of this.#unread) this.#add(JSON.parse(splice) as Splice);
    this.#unread = [];
    return this.#sources.has(src);
  }

  #add(splice: Splice): void {
    for (const src of imageSources(splice.nodes)) this.#sources.add(src);
  }
}
