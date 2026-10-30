// Where each version of the spec goes in the site (website/docs/design.md, decision 11). Until version 1 is released,
// `spec/` is the draft from `main`. Each release tag `spec-vX.Y` adds `spec/X.Y/`, the spec as it was at the tag; then
// `spec/` shows the latest release, and `spec/draft/` the draft.

/** A page of the spec. */
export interface SpecPage {
  /** The folder of the site it goes in, ending in `/`. */
  folder: string;
  /** Its version, `1.0`; absent for the draft. */
  version?: string;
  /** The git ref its text comes from: the tag, or `undefined` for the working tree. */
  ref?: string;
}

const TAG = /^spec-v(\d+)\.(\d+)$/;

/** The released versions among `tags`, newest first; other tags are left out. */
export function releases(tags: readonly string[]): { version: string; tag: string }[] {
  return tags
    .map((tag) => ({ tag, parts: TAG.exec(tag) }))
    .filter((found): found is { tag: string; parts: RegExpExecArray } => found.parts !== null)
    .map(({ tag, parts }) => ({ tag, major: Number(parts[1]), minor: Number(parts[2]) }))
    .sort((a, b) => b.major - a.major || b.minor - a.minor)
    .map(({ tag, major, minor }) => ({ tag, version: `${String(major)}.${String(minor)}` }));
}

/** Every page of the spec the site has, given the repository's tags: the draft, and each release. */
export function specPages(tags: readonly string[]): SpecPage[] {
  const released = releases(tags);
  const latest = released[0];
  if (!latest) return [{ folder: "spec/" }];
  return [
    { folder: "spec/", version: latest.version, ref: latest.tag },
    { folder: "spec/draft/" },
    ...released.map(({ version, tag }) => ({ folder: `spec/${version}/`, version, ref: tag })),
  ];
}

/** The path from a page in `folder` to the site's root: `../` per folder. */
export function rootOf(folder: string): string {
  return "../".repeat(folder.split("/").filter(Boolean).length);
}
