// Which release a cache entry is (audit S4): a pack's name and version as well as its integrity, so a release one pack
// fetched is never given as another pack's, whatever integrity an import pins.

/** A pack's release, as the cache keeps its files: the pack, the version and the integrity of its tarball. */
export interface ReleaseId {
  /** The pack's name, `@kit/charts`. */
  pack: string;
  /** The exact version, `1.0.0`. */
  version: string;
  /** The tarball's integrity, `sha512-…`. */
  integrity: string;
}

/**
 * `release` as one string, for a key of a cache's own: two releases have the same key only when their pack, version
 * and integrity are all the same.
 *
 * @example
 * releaseKey({ pack: "@kit/charts", version: "1.0.0", integrity: "sha512-E" }); // "@kit/charts@1.0.0#sha512-E"
 */
export const releaseKey = ({ pack, version, integrity }: ReleaseId): string => `${pack}@${version}#${integrity}`;
