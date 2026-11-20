// A pack's placeholder (RELEASING.md, *A placeholder first*): a `0.0.0` with no code, which creates the package on
// npm, so its trusted publisher can be set and `release-packs.yml` can stage its first real version.

/** The scope of this repository's packs. */
export const SCOPE = "@comp-markdown";
/** The repository the packs are released from, as npm's trusted publishers name it. */
export const REPOSITORY = "comp-markdown/compmd";
/** The workflow that stages every pack's releases (docs/releases.md, decision 8). */
export const WORKFLOW = "release-packs.yml";

/** Whether `pack` is a pack's name: lowercase letters and digits, in words joined by `-`, as `embed-youtube`. */
export const isPackName = (pack: string): boolean => /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(pack);

/** The npm package of `pack`: `@comp-markdown/card`. */
export const packageOf = (pack: string): string => `${SCOPE}/${pack}`;

/**
 * The placeholder's files, by name: a `package.json` that says what it is, a README pointing to the real versions,
 * and `license`, the MIT licence the pack's code is under.
 *
 * @example
 * placeholderFiles("card", mit)["package.json"]; // {"name": "@comp-markdown/card", "version": "0.0.0", …}
 */
export function placeholderFiles(pack: string, license: string): Record<string, string> {
  const name = packageOf(pack);
  const folder = `https://github.com/${REPOSITORY}/tree/main/compmd-packs/${pack}`;
  // The repository is the one the trusted publisher names: npm's provenance checks the two match.
  const manifest = {
    name,
    version: "0.0.0",
    description: `A placeholder with no code, which created ${name} on npm: install a later version.`,
    keywords: ["compmd", "compmd-pack"],
    homepage: "https://compmd.dev/",
    repository: { type: "git", url: `git+https://github.com/${REPOSITORY}.git`, directory: `compmd-packs/${pack}` },
    license: "MIT",
    files: ["README.md", "LICENSE"],
  };
  const readme = `# ${name}

A placeholder with no code. It created the package on npm, so that its versions can be published from [its folder in the CompMD repository](${folder}), by trusted publishing, with their provenance.

Install a later version: \`npm install ${name}@latest\`.
`;
  return { "package.json": `${JSON.stringify(manifest, null, 2)}\n`, "README.md": readme, LICENSE: license };
}
