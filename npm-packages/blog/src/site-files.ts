// A built blog's files, each by its path in the site and what wrote it, so two never write one path (audit B16): tags
// `C` and `C++` both slug to `c`, a post named `tags` is the tags' page's folder, a post's file `feed.xml` is the feed.

import { BlogError } from "./config.ts";

/** A built blog's files, by their paths in the site, each written once. */
export class SiteFiles {
  readonly #files = new Map<string, string | Uint8Array>();
  readonly #owners = new Map<string, string>();

  /**
   * Writes `content` at `path`, for `owner`: what it is, as the error names it (`the page of the tag "C"`).
   *
   * @throws BlogError naming both owners when another wrote `path` already.
   */
  set(path: string, content: string | Uint8Array, owner: string): void {
    const before = this.#owners.get(path);
    if (before !== undefined) throw new BlogError(`${path} would be written twice: by ${before}, and by ${owner}; rename one of them`);
    this.#owners.set(path, owner);
    this.#files.set(path, content);
  }

  /** The files, by their paths in the site, in the order they were written. */
  get files(): Map<string, string | Uint8Array> {
    return new Map(this.#files);
  }
}
