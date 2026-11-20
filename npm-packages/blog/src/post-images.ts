// A post's images at build time (spec §15): an image of the post's own files, or from an origin the blog reaches
// without asking (`origins.ts`), shows; any other shows its alt text, with the button a live page's script answers,
// so the built page loads no image the live one would ask a reader about.

import type { ImageSource, Refused } from "@compmd/render";

/** Where a relative `src` resolves while the origin is worked out: no image is ever fetched from it. */
const OWN = "https://post.invalid/";

/** The origin of `src` when it is another site's image, `https:` or `http:`; `undefined` for the post's own files. */
function remoteOrigin(src: string): string | undefined {
  try {
    const url = new URL(src, OWN);
    return (url.protocol === "https:" || url.protocol === "http:") && url.origin !== new URL(OWN).origin ? url.origin : undefined;
  } catch {
    return undefined;
  }
}

/** The images of one post's render: what it shows, and the origins it refused. */
export class PostImages {
  /** The origins of the images it refused, for the build's warnings. */
  readonly refused = new Set<string>();
  readonly #allowed: readonly string[] | "any";
  readonly #answers: Promise<unknown>[] = [];

  /** @param allowed The origins the blog reaches without asking, or `"any"`. */
  constructor(allowed: readonly string[] | "any") {
    this.#allowed = allowed;
  }

  /** The renderer's image source: an image's `src` when it shows, its origin refused otherwise. */
  readonly source: ImageSource = (src) => {
    const answer = Promise.resolve(this.#answer(src));
    this.#answers.push(answer);
    return answer;
  };

  /** Resolves once every image asked about has its answer, and the renderer has drawn it. */
  async settled(): Promise<void> {
    await Promise.all(this.#answers);
  }

  #answer(src: string): string | Refused {
    const origin = remoteOrigin(src);
    if (origin === undefined || this.#allowed === "any" || this.#allowed.includes(origin)) return src;
    this.refused.add(origin);
    return { refused: origin };
  }
}
