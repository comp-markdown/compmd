// The page's script of one blog, kept between builds while a dev server runs (docs/blog-tool.md, decision 3).

import type { Client } from "./live.ts";

/**
 * The page's script of one blog, kept while a dev server runs: building it takes seconds, so each build reuses it,
 * until the server says that what it is made from changed. A dev server makes one for its blog and gives it to each
 * build (`BuildOptions.clients`).
 *
 * @example
 * const clients = new ClientCache();
 * await buildSite(dir, { drafts: true, clients });   // builds the script
 * await buildSite(dir, { drafts: true, clients });   // reuses it
 * clients.forget();                                  // compmd.config.json changed
 */
export class ClientCache {
  #client: Promise<Client> | undefined;

  /** The script: the one kept, or the one `build` makes, kept for the next time. A build that fails isn't kept. */
  get(build: () => Promise<Client>): Promise<Client> {
    if (this.#client) return this.#client;
    const building = build();
    this.#client = building;
    building.catch(() => {
      if (this.#client === building) this.#client = undefined;
    });
    return building;
  }

  /** Forgets the script, so the next build makes it again. */
  forget(): void {
    this.#client = undefined;
  }
}
