// A page frame's images (frame mode): the frame has no network, so the host's page fetches each image of the trees it
// sent the frame, and gives it as a Blob, which the frame shows from a `blob:` URL. Its policy stays as it is, so code in
// the frame still loads nothing itself; and since that code could ask too, the host's page gives only those images.
// An image of one of the host's own schemes (spec §15) is the host's to load: its file comes from the host, in both modes.

import { imageSources, urlScheme, type ExpandedRoot, type ImageSource, type Refused, type Splice } from "@compmd/render";
import { ImageGate } from "./image-gate.ts";
import { originOf } from "./origins.ts";

/** How long a frame may not ask about an origin again after the reader refused it, in ms (docs/remote-images.md, decision 6). */
export const ASK_AGAIN_MS = 60_000;

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
  readonly #gate: ImageGate;
  /** The images' `src`, as the frame asks for them. */
  #sources = new Set<string>();
  /** Splices as a processor's JSON text, read only once the frame asks for an image they may have. */
  #unread: string[] = [];
  /** When the reader last refused an origin the frame asked about, by origin. */
  readonly #refusedAt = new Map<string, number>();

  /**
   * Fetches with `fetch`, a relative `src` against `base`, as an image in the host's page would load, once `gate` lets
   * it (spec §15); an image of one of the host's own schemes comes from `own.file`, never fetched.
   */
  constructor(fetcher: typeof fetch, base: string, own: OwnImages = { schemes: [] }, gate: ImageGate = new ImageGate()) {
    // Called on its own: the page's `fetch` throws when called on another object.
    this.#fetch = (input, init) => fetcher(input, init);
    this.#base = base;
    this.#own = own;
    this.#gate = gate;
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
   * The file of the image with `src`, `undefined` when it isn't one of the frame's, can't be had, or isn't an image, or
   * `{ refused: origin }` when the reader hasn't allowed its origin (asked the first time). One of the host's own
   * schemes comes from the host; any other is fetched without credentials and without a `Referer`.
   */
  async file(src: string): Promise<Blob | Refused | undefined> {
    if (!this.#has(src)) return undefined;
    try {
      if (isOwn(src, this.#own.schemes)) return imageOnly(await this.#own.file?.(src));
      const decision = await this.#gate.decide(src);
      if ("refused" in decision) return decision;
      return imageOnly(await this.#fetched(src));
    } catch {
      return undefined;
    }
  }

  /**
   * The frame asks for the reader to be asked about `origin`, from the button beside a refused image. Code in the
   * frame may have asked instead, so the reader is asked only about an origin of the frame's images, and not again
   * within `ASK_AGAIN_MS` of refusing it.
   */
  async allowOrigin(origin: string, now: () => number = Date.now): Promise<void> {
    this.#readSplices();
    if (![...this.#sources].some((src) => originOf(src) === origin)) return;
    const refused = this.#refusedAt.get(origin);
    if (refused !== undefined && now() - refused < ASK_AGAIN_MS) return;
    if (!(await this.#gate.allow(origin))) this.#refusedAt.set(origin, now());
  }

  async #fetched(src: string): Promise<Blob | undefined> {
    const response = await this.#fetch(new URL(src, this.#base), { credentials: "omit", referrerPolicy: "no-referrer" });
    return response.ok ? response.blob() : undefined;
  }

  #has(src: string): boolean {
    if (this.#sources.has(src)) return true;
    this.#readSplices();
    return this.#sources.has(src);
  }

  /** Adds the images of the splices given as text, which were left unread until needed. */
  #readSplices(): void {
    for (const splice of this.#unread) this.#add(JSON.parse(splice) as Splice);
    this.#unread = [];
  }

  #add(splice: Splice): void {
    for (const src of imageSources(splice.nodes, this.#own.schemes)) this.#sources.add(src);
  }
}

/** `file` when it is an image; a file of another type isn't shown. */
const imageOnly = (file: Blob | undefined): Blob | undefined => (file?.type.startsWith("image/") ? file : undefined);

/**
 * Host mode's images: one of the host's own schemes from a `blob:` URL of the file `file` gives, asked for once while
 * it has one, or from its `src` without `file`; any other from its `src`, as the host's page loads it, once `gate`
 * lets it (spec §15).
 *
 * @example
 * createPage(container, { ownSchemes: ["app"], images: hostImages(["app"], (src) => attachment(src), gate) });
 */
export function hostImages(schemes: readonly string[], file: ((src: string) => Promise<Blob | undefined>) | undefined, gate: ImageGate): ImageSource {
  const shown = new Map<string, Promise<string | undefined>>();
  return async (src) => {
    if (!isOwn(src, schemes)) {
      const decision = await gate.decide(src);
      return "refused" in decision ? decision : src;
    }
    if (!file) return src;
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
