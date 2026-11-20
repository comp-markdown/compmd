// The packs to release: those whose version in versions.json npm doesn't have (docs/releases.md, decision 8), each
// with its release's tag, `<pack>-v<version>`.

import { packageOf } from "./placeholder.ts";
import type { Versions } from "./versions.ts";

/** A pack's release. */
export interface Release {
  pack: string;
  /** Its npm package: `@comp-markdown/card`. */
  name: string;
  version: string;
  /** Its tag: `card-v0.1.0`. */
  tag: string;
}

/** A pack's release's tag: `card-v0.1.0`. */
export const tagOf = (pack: string, version: string): string => `${pack}-v${version}`;

/**
 * The releases of `versions` npm doesn't have, in the packs' order, asking `has` for each.
 *
 * @example
 * await unreleased({ card: "0.1.0" }, async () => false); // [{ pack: "card", name: "@comp-markdown/card", version: "0.1.0", tag: "card-v0.1.0" }]
 */
export async function unreleased(versions: Versions, has: (name: string, version: string) => Promise<boolean>): Promise<Release[]> {
  const releases: Release[] = [];
  for (const [pack, version] of Object.entries(versions).sort(([a], [b]) => a.localeCompare(b))) {
    const name = packageOf(pack);
    if (!(await has(name, version))) releases.push({ pack, name, version, tag: tagOf(pack, version) });
  }
  return releases;
}
