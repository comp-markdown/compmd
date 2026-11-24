// A document's remote images, asked about (spec §15, docs/remote-images.md): every image the page draws passes this
// gate, in both page modes. An image on the page's own origin, one of the host's own schemes and one of an origin the
// host configured pass; one of an origin the reader allowed passes; any other is asked about the first time it is
// drawn, with the same answers as the document's data (`origins.ts`), and is refused, showing its alt text, until its
// origin is allowed.

import { answerFor, askAbout, askWithDialog, localAnswers, type AskOrigin, type ConfiguredOrigins, type OriginAnswers } from "./origins.ts";
import { remoteOrigin } from "./remote-origin.ts";

/** Who asks the reader about images' origins, and where the answers are kept: what a host gives `loadData` too. */
export interface ImageGateOptions {
  /**
   * The origins the host configured, which need no question; or `"any"`, every origin, for a host's own documents,
   * such as a static site's or a blog's (spec §15): then no image is asked about.
   */
  configured?: ConfiguredOrigins;
  /** What the question calls the document: its title or path. */
  document?: string;
  /** How the reader is asked; a plain dialog by default. A host without a reader gives one that refuses. */
  ask?: AskOrigin;
  /** Where the answers are kept; `localStorage` by default, shared with the data's. */
  answers?: OriginAnswers;
  /**
   * The URL the page reads images' `src` against, as the browser loads them: the host's page's `document.baseURI` by
   * default. Outside a page it must be given.
   */
  base?: string;
}

/** The host's page's base URL. */
function pageBase(): string {
  if (typeof document === "undefined") throw new Error("an ImageGate outside a page needs a `base` to read images' `src` against");
  return document.baseURI;
}

/** What the gate says of an image: show it, or it is refused for its origin. */
export type ImageDecision = { show: true } | { refused: string };

/**
 * The gate a page's images pass.
 *
 * @example
 * const gate = new ImageGate({ configured: ["https://cdn.example.com"] }); // in the host's page, read against its base
 * await gate.decide("https://photos.example.com/a.jpg"); // asks the reader, then { show: true } or { refused: … }
 * gate.onAllowed((origin) => page.imagesAllowed(origin));
 */
export class ImageGate {
  readonly #configured: ReadonlySet<string> | "any";
  readonly #document: string;
  readonly #ask: AskOrigin;
  readonly #answers: OriginAnswers;
  readonly #base: string;
  readonly #listeners = new Set<(origin: string) => void>();

  /** @throws Error outside a page, without `options.base`. */
  constructor(options: ImageGateOptions = {}) {
    this.#configured = options.configured === "any" ? "any" : new Set(options.configured ?? []);
    this.#document = options.document ?? "this document";
    this.#ask = options.ask ?? askWithDialog;
    this.#answers = options.answers ?? localAnswers();
    this.#base = options.base ?? pageBase();
  }

  /**
   * Whether the image `src` shows: one on the page's own origin, `src` read against the gate's `base` as the page loads
   * it (`remoteOrigin`), one of another scheme than `http:` and `https:` (the renderer lets only the host's own
   * through), and one of a configured or allowed origin do; for any other origin the reader is asked, the first time,
   * and it is refused until they allow it.
   */
  async decide(src: string): Promise<ImageDecision> {
    const origin = remoteOrigin(src, this.#base);
    if (origin === undefined || this.#isConfigured(origin)) return { show: true };
    const answer = await answerFor(origin, ["images"], this.#document, this.#answers, this.#ask);
    return answer === "allow" ? { show: true } : { refused: origin };
  }

  /**
   * The reader asks to see the images of `origin`, from the button beside a refused one in the host's page: their
   * answer is forgotten and they are asked again; when they allow it, each listener hears of it. Resolves to whether
   * they did.
   */
  async allow(origin: string): Promise<boolean> {
    if (this.#isConfigured(origin)) return true;
    this.#answers.forget(origin);
    return this.#asked(origin);
  }

  /**
   * A frame asks for the reader to be asked about `origin` again, from its button beside a refused image, or its code
   * does: they are asked, and the answer they gave before stands until they give a new one, so a frame can't erase it.
   * When they allow it, each listener hears of it. Resolves to whether they did.
   */
  async askAgain(origin: string): Promise<boolean> {
    if (this.#isConfigured(origin)) return true;
    return this.#asked(origin);
  }

  /** Asks the reader about `origin` now, telling each listener when they allow it. */
  async #asked(origin: string): Promise<boolean> {
    const allowed = (await askAbout(origin, ["images"], this.#document, this.#answers, this.#ask)) === "allow";
    if (allowed) for (const listener of this.#listeners) listener(origin);
    return allowed;
  }

  /** Whether the host configured `origin`, by name or by allowing every origin. */
  #isConfigured(origin: string): boolean {
    return this.#configured === "any" || this.#configured.has(origin);
  }

  /** Calls `listener` with each origin the reader allows from now on; gives what stops it. */
  onAllowed(listener: (origin: string) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }
}
