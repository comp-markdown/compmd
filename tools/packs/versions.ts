// The packs' versions (docs/releases.md, decision 8): `compmd-packs/versions.json` is the one place a pack's version is
// written; each pack's `package.json` and its document's `export.version` follow it.

import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

/** The packs' folder, and the file of their versions in it, relative to the repository. */
export const PACKS = "compmd-packs";
export const VERSIONS = `${PACKS}/versions.json`;

/** Each pack's version, by its folder's name: `{ "mermaid": "0.1.2" }`. */
export type Versions = Record<string, string>;

/** A pack as its files have it. */
export interface Pack {
  /** Its folder's name in `compmd-packs/`, and its package's name in the scope. */
  name: string;
  /** Its `package.json`, and its document, `index.comp.md`, as text. */
  manifest: string;
  document: string;
}

/** A semantic version: `0.1.2`, `1.0.0-beta.1`. */
const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;
/** `package.json`'s `"version": "…"`, its first, which is the package's own. */
const MANIFEST_VERSION = /("version":\s*)"([^"]*)"/;
/** The document's `export.version`, in its frontmatter: the first `version:` after `export:`, indented under it. */
const EXPORT_VERSION = /^([ \t]*export:[ \t]*\n(?:[ \t]+.*\n)*?[ \t]+version:[ \t]*)"([^"\n]*)"/m;

/** `package.json`'s version, or `undefined` when it has none. */
export const manifestVersion = (manifest: string): string | undefined => MANIFEST_VERSION.exec(manifest)?.[2];

/** The document's `export.version`, or `undefined` when it has none. */
export const documentVersion = (document: string): string | undefined => EXPORT_VERSION.exec(document)?.[2];

/** The pack's files with `version` written in both, and everything else as it was. */
export function withVersion(pack: Pack, version: string): Pack {
  return {
    ...pack,
    manifest: pack.manifest.replace(MANIFEST_VERSION, (_, key: string) => `${key}"${version}"`),
    document: pack.document.replace(EXPORT_VERSION, (_, key: string) => `${key}"${version}"`),
  };
}

/** What versions.json and the packs disagree on, which no write can fix: a pack missing on one side, a bad version. */
export function errors(versions: Versions, packs: readonly Pack[]): string[] {
  const found: string[] = [];
  const names = new Set(packs.map((pack) => pack.name));
  for (const pack of packs) if (!(pack.name in versions)) found.push(`${PACKS}/${pack.name} has no version in ${VERSIONS}`);
  for (const [name, version] of Object.entries(versions)) {
    if (!names.has(name)) found.push(`${VERSIONS} names ${name}, which isn't a pack in ${PACKS}/`);
    else if (!SEMVER.test(version)) found.push(`${VERSIONS} gives ${name} "${version}", which isn't a version`);
  }
  for (const pack of packs) {
    if (manifestVersion(pack.manifest) === undefined) found.push(`${PACKS}/${pack.name}/package.json has no version`);
    if (documentVersion(pack.document) === undefined) found.push(`${PACKS}/${pack.name}/index.comp.md has no export.version`);
  }
  return found;
}

/** The packs whose files don't have their version in versions.json, with what they have. */
export function differing(versions: Versions, packs: readonly Pack[]): { pack: Pack; version: string; written: string[] }[] {
  return packs.flatMap((pack) => {
    const version = versions[pack.name];
    if (version === undefined) return [];
    const written = [manifestVersion(pack.manifest), documentVersion(pack.document)].filter((found): found is string => found !== undefined && found !== version);
    return written.length > 0 ? [{ pack, version, written }] : [];
  });
}

/** The packs and their versions in the repository at `root`: each folder of `compmd-packs/` with a `package.json`. */
export function readPacks(root: string): { versions: Versions; packs: Pack[] } {
  const folder = join(root, PACKS);
  const versions = JSON.parse(readFileSync(join(root, VERSIONS), "utf8")) as Versions;
  const packs = readdirSync(folder, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && existsSync(join(folder, entry.name, "package.json")))
    .map((entry) => ({
      name: entry.name,
      manifest: readFileSync(join(folder, entry.name, "package.json"), "utf8"),
      document: existsSync(join(folder, entry.name, "index.comp.md")) ? readFileSync(join(folder, entry.name, "index.comp.md"), "utf8") : "",
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
  return { versions, packs };
}

/** Writes `pack`'s files into the repository at `root`. */
export function writePack(root: string, pack: Pack): void {
  writeFileSync(join(root, PACKS, pack.name, "package.json"), pack.manifest);
  writeFileSync(join(root, PACKS, pack.name, "index.comp.md"), pack.document);
}

/** `cargo make pack-versions`, which `ci` runs: fails when versions.json and the packs disagree. */
export function check(root: string): number {
  const { versions, packs } = readPacks(root);
  const found = [
    ...errors(versions, packs),
    ...differing(versions, packs).map(({ pack, version, written }) => `${PACKS}/${pack.name} has ${written.join(" and ")}, ${VERSIONS} ${version}: \`cargo make release-packs\` writes it`),
  ];
  for (const problem of found) console.error(problem);
  return found.length === 0 ? 0 : 1;
}

if (import.meta.main) process.exitCode = check(join(import.meta.dirname, "../.."));
