// A page frame's images (frame mode): the frame has no network, so the host's page fetches each image of the trees it
// sent the frame, and gives it as a Blob, which the frame shows from a `blob:` URL. Its policy stays as it is, so code in
// the frame still loads nothing itself; and since that code could ask too, the host's page gives only those images.
// An image of one of the host's own schemes (spec §15) is the host's to load: its file comes from the host, in both modes.
// The files of the document's own fonts (spec §9.4) come the same way, from the host's page's origin only.

import { imageSources, urlScheme, type ExpandedRoot, type FontSource, type ImageSource, type Refused, type Splice } from "@compmd/render";
import { packFileOf } from "./found.ts";
import { ImageGate } from "./image-gate.ts";
import { remoteOrigin } from "./remote-origin.ts";
import { SentFonts } from "./sent-fonts.ts";
import { SentNodes } from "./sent-nodes.ts";

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
 * Where the host serves the packs' files (`at`, ending in `/`), and the files a frame may load that it holds of packs
 * fetched from a registry while the document is open, images and fonts' files, by their place, which nothing serves at
 * `at` (spec §6.4, §9.4, §12.3).
 */
export interface PackFiles {
  at: string;
  files?: ((place: string) => Blob | undefined) | undefined;
}

/** The registry pack's file a pack's image's or font's `src` (its URL under `packs.at`) names, if the host holds it. */
const heldFile = (src: string, packs: PackFiles | undefined): Blob | undefined =>
  packs?.files && src.startsWith(packs.at) ? packs.files(src.slice(packs.at.length)) : undefined;

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
  readonly #packs: PackFiles | undefined;
  /** The images' `src`, as the frame asks for them. */
  readonly #sources: SentNodes<string>;
  /** The files of the fonts of the document's own, which the frame asks for as it asks for images (spec §9.4). */
  readonly #fonts: SentFonts;
  /** When the reader last refused an origin the frame asked about, by origin. */
  readonly #refusedAt = new Map<string, number>();

  /**
   * Fetches with `fetch`, a relative `src` against `base`, as an image in the host's page would load, once `gate` lets
   * it (spec §15); an image of one of the host's own schemes comes from `own.file`, never fetched; a pack's image, by
   * its URL under `packs.at`, from `packs.files` when the host holds it (a registry pack's), else fetched.
   */
  constructor(fetcher: typeof fetch, base: string, own: OwnImages = { schemes: [] }, gate: ImageGate = new ImageGate({ base }), packs?: PackFiles) {
    // Called on its own: the page's `fetch` throws when called on another object.
    this.#fetch = (input, init) => fetcher(input, init);
    this.#base = base;
    this.#own = own;
    this.#gate = gate;
    this.#packs = packs;
    // A pack's image by the URL the frame asks for, which the frame makes from `packs.at` too (spec §12.3).
    const packFile = packs && packFileOf(packs.at);
    this.#sources = new SentNodes((nodes) => imageSources(nodes, own.schemes, packFile));
    this.#fonts = new SentFonts(packFile);
  }

  /** The frame renders `tree`: its images and fonts are the frame's now. */
  rendered(tree: ExpandedRoot): void {
    this.#sources.rendered(tree.children);
    this.#fonts.rendered(tree.fonts ?? []);
  }

  /** The frame applies `splices`: their images and fonts are the frame's too. */
  spliced(splices: readonly (Splice | string)[]): void {
    this.#sources.spliced(splices);
    this.#fonts.spliced(splices);
  }

  /**
   * The file of the image with `src`, `undefined` when it isn't one of the frame's, can't be had, or isn't an image, or
   * `{ refused: origin }` when the reader hasn't allowed its origin (asked the first time). One of the host's own
   * schemes comes from the host; any other is fetched without credentials and without a `Referer`.
   */
  async file(src: string): Promise<Blob | Refused | undefined> {
    if (this.#fonts.has(src)) return this.#fontFile(src);
    if (!this.#sources.has(src)) return undefined;
    try {
      if (isOwn(src, this.#own.schemes)) return imageOnly(await this.#own.file?.(src));
      const held = heldFile(src, this.#packs);
      if (held) return imageOnly(held);
      // The URL fetched is the one decided on.
      const url = new URL(src, this.#base);
      const decision = await this.#gate.decide(url.href);
      if ("refused" in decision) return decision;
      return imageOnly(await this.#fetched(url));
    } catch {
      return undefined;
    }
  }

  /**
   * The frame asks for the reader to be asked about `origin`, from the button beside a refused image. Code in the
   * frame may have asked instead, so the reader is asked only about the origin an image of the frame's loads from (its
   * `src` read against the base), not again within `ASK_AGAIN_MS` of refusing it, and the answer they gave before
   * stands until they give a new one (`ImageGate.askAgain`).
   */
  async allowOrigin(origin: string, now: () => number = Date.now): Promise<void> {
    if (!this.#sources.some((src) => remoteOrigin(src, this.#base) === origin)) return;
    const refused = this.#refusedAt.get(origin);
    if (refused !== undefined && now() - refused < ASK_AGAIN_MS) return;
    if (!(await this.#gate.askAgain(origin))) this.#refusedAt.set(origin, now());
  }

  /**
   * A font's file: a registry pack's the host holds, or fetched, never from another origin than the host's page, since
   * a font is never remote (spec §9.4).
   */
  async #fontFile(src: string): Promise<Blob | undefined> {
    const held = heldFile(src, this.#packs);
    if (held) return held;
    if (new URL(src, this.#base).origin !== new URL(this.#base).origin) return undefined;
    try {
      return await this.#fetched(new URL(src, this.#base));
    } catch {
      return undefined;
    }
  }

  async #fetched(url: URL): Promise<Blob | undefined> {
    const response = await this.#fetch(url, { credentials: "omit", referrerPolicy: "no-referrer" });
    return response.ok ? response.blob() : undefined;
  }
}

/** `file` when it is an image; a file of another type isn't shown. */
const imageOnly = (file: Blob | undefined): Blob | undefined => (file?.type.startsWith("image/") ? file : undefined);

/**
 * Host mode's fonts of the document's own: a registry pack's file the host holds (`packs.files`) from a `blob:` URL of
 * it, made once; any other from its `src`, as the host's page loads it.
 *
 * @example
 * createPage(container, { fonts: hostFonts({ at: "/packs/", files: (place) => held.get(place) }) });
 */
export function hostFonts(packs: PackFiles | undefined): FontSource {
  const shown = new Map<string, string>();
  return (src) => {
    const held = heldFile(src, packs);
    if (!held) return Promise.resolve(src);
    let url = shown.get(src);
    if (url === undefined) shown.set(src, (url = URL.createObjectURL(held)));
    return Promise.resolve(url);
  };
}

/**
 * Host mode's images: one of the host's own schemes from a `blob:` URL of the file `file` gives, asked for once while
 * it has one, or from its `src` without `file`; a registry pack's image the host holds (`packs.files`) from a `blob:`
 * URL of it; any other from its `src`, as the host's page loads it, once `gate` lets it (spec §15).
 *
 * @example
 * createPage(container, { ownSchemes: ["app"], images: hostImages(["app"], (src) => attachment(src), gate) });
 */
export function hostImages(schemes: readonly string[], file: ((src: string) => Promise<Blob | undefined>) | undefined, gate: ImageGate, packs?: PackFiles): ImageSource {
  const shown = new Map<string, Promise<string | undefined>>();
  /** `src` from a `blob:` URL of the file `get` gives, asked for once while it has one. */
  const fromBlob = (src: string, get: () => Promise<Blob | undefined>): Promise<string | undefined> => {
    let url = shown.get(src);
    if (!url) {
      url = get().then(
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
  return async (src) => {
    const held = imageOnly(heldFile(src, packs));
    if (held) return fromBlob(src, () => Promise.resolve(held));
    if (!isOwn(src, schemes)) {
      const decision = await gate.decide(src);
      return "refused" in decision ? decision : src;
    }
    if (!file) return src;
    return fromBlob(src, () => file(src));
  };
}
