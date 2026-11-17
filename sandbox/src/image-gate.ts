// A document's remote images, asked about (spec §15, docs/remote-images.md): every image the page draws passes this
// gate, in both page modes. A relative `src`, one of the host's own schemes and one of an origin the host configured
// pass; one of an origin the reader allowed passes; any other is asked about the first time it is drawn, with the same
// answers as the document's data (`origins.ts`), and is refused, showing its alt text, until its origin is allowed.

import { urlScheme } from "@compmd/render";
import { answerFor, askWithDialog, localAnswers, originOf, type AskOrigin, type OriginAnswers } from "./origins.ts";

/** Who asks the reader about images' origins, and where the answers are kept: what a host gives `loadData` too. */
export interface ImageGateOptions {
  /** The origins the host configured, which need no question. */
  configured?: readonly string[];
  /** What the question calls the document: its title or path. */
  document?: string;
  /** How the reader is asked; a plain dialog by default. A host without a reader gives one that refuses. */
  ask?: AskOrigin;
  /** Where the answers are kept; `localStorage` by default, shared with the data's. */
  answers?: OriginAnswers;
}

/** What the gate says of an image: show it, or it is refused for its origin. */
export type ImageDecision = { show: true } | { refused: string };

/**
 * The gate a page's images pass.
 *
 * @example
 * const gate = new ImageGate({ configured: ["https://cdn.example.com"] });
 * await gate.decide("https://photos.example.com/a.jpg"); // asks the reader, then { show: true } or { refused: … }
 * gate.onAllowed((origin) => page.imagesAllowed(origin));
 */
export class ImageGate {
  readonly #configured: ReadonlySet<string>;
  readonly #document: string;
  readonly #ask: AskOrigin;
  readonly #answers: OriginAnswers;
  readonly #listeners = new Set<(origin: string) => void>();

  constructor(options: ImageGateOptions = {}) {
    this.#configured = new Set(options.configured ?? []);
    this.#document = options.document ?? "this document";
    this.#ask = options.ask ?? askWithDialog;
    this.#answers = options.answers ?? localAnswers();
  }

  /**
   * Whether the image `src` shows: a relative one, one of another scheme than `http:` and `https:` (the renderer lets
   * only the host's own through), and one of a configured or allowed origin do; for any other origin the reader is
   * asked, the first time, and it is refused until they allow it.
   */
  async decide(src: string): Promise<ImageDecision> {
    const scheme = urlScheme(src);
    if (scheme !== "http" && scheme !== "https") return { show: true };
    const origin = originOf(src);
    if (origin === undefined || this.#configured.has(origin)) return { show: true };
    const answer = await answerFor(origin, ["images"], this.#document, this.#answers, this.#ask);
    return answer === "allow" ? { show: true } : { refused: origin };
  }

  /**
   * The reader asks to see the images of `origin`, from the button beside a refused one: their answer is forgotten and
   * they are asked again; when they allow it, each listener hears of it. Resolves to whether they did.
   */
  async allow(origin: string): Promise<boolean> {
    if (this.#configured.has(origin)) return true;
    this.#answers.forget(origin);
    const allowed = (await answerFor(origin, ["images"], this.#document, this.#answers, this.#ask)) === "allow";
    if (allowed) for (const listener of this.#listeners) listener(origin);
    return allowed;
  }

  /** Calls `listener` with each origin the reader allows from now on; gives what stops it. */
  onAllowed(listener: (origin: string) => void): () => void {
    this.#listeners.add(listener);
    return () => this.#listeners.delete(listener);
  }
}
