// What a pack's documents reach (spec §8.6): its document's references, and those of each document it imports, in
// turn. What a folder pack without `files` publishes, and what the publish check finds missing.

import { referencesOf, type References } from "./references.ts";

/** What a pack's documents reach, each by its path in the pack, with the documents' texts. */
export interface Reach extends References {
  /** Each document read, by its path in the pack, with its text. */
  documents: Map<string, string>;
}

/**
 * What the documents reach from `document`, reading each with `read`; a document `read` has no text for is left for
 * the caller to report.
 *
 * @example
 * const reach = await reachOf("index.comp.md", (path) => readFile(join(dir, path), "utf8").catch(() => undefined));
 * reach.fonts; // ["fonts/harbour.woff2"]
 */
export async function reachOf(document: string, read: (path: string) => Promise<string | undefined>): Promise<Reach> {
  const reach: Reach = { imports: [], data: [], fonts: [], destinations: [], outside: [], unlisted: [], documents: new Map() };
  const queue = [document];
  for (let path = queue.shift(); path !== undefined; path = queue.shift()) {
    if (reach.documents.has(path)) continue;
    const text = await read(path);
    if (text === undefined) continue;
    reach.documents.set(path, text);
    const references = referencesOf(path, text);
    for (const kind of ["imports", "data", "fonts", "destinations", "outside", "unlisted"] as const) reach[kind].push(...references[kind].filter((found) => !reach[kind].includes(found)));
    queue.push(...references.imports.filter((imported) => imported.endsWith(".comp.md")));
  }
  return reach;
}
