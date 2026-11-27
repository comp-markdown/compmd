// A placeholder (RELEASING.md, *A placeholder first*): a `0.0.0` with no code, which creates a package on npm, so its
// trusted publisher can be set and a release workflow can stage its first real version. The packs' and the
// implementation's packages share it, each with its scope, its workflow and its folder in the repository.

/** The scope of this repository's packs. */
export const SCOPE = "@comp-markdown";
/** The repository the packages are released from, as npm's trusted publishers name it. */
export const REPOSITORY = "comp-markdown/compmd";
/** The workflow that stages every pack's releases (docs/releases.md, decision 8). */
export const WORKFLOW = "release-packs.yml";

/** Whether `pack` is a pack's name: lowercase letters and digits, in words joined by `-`, as `embed-youtube`. */
export const isPackName = (pack: string): boolean => /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(pack);

/** The npm package of `pack`: `@comp-markdown/card`. */
export const packageOf = (pack: string): string => `${SCOPE}/${pack}`;

/** What a placeholder is of: the package's name, its folder in the repository, and its keywords. */
export interface PlaceholderOf {
  name: string;
  directory: string;
  keywords: readonly string[];
}

/** What a pack's placeholder is of: its package, in `compmd-packs/<pack>`. */
export const packPlaceholder = (pack: string): PlaceholderOf => ({
  name: packageOf(pack),
  directory: `compmd-packs/${pack}`,
  keywords: ["compmd", "compmd-pack"],
});

/**
 * The placeholder's files, by name: a `package.json` that says what it is, a README pointing to the real versions,
 * and `license`, the MIT licence the package's code is under.
 *
 * @example
 * placeholderFiles(packPlaceholder("card"), mit)["package.json"]; // {"name": "@comp-markdown/card", "version": "0.0.0", …}
 */
export function placeholderFiles({ name, directory, keywords }: PlaceholderOf, license: string): Record<string, string> {
  const folder = `https://github.com/${REPOSITORY}/tree/main/${directory}`;
  // The repository is the one the trusted publisher names: npm's provenance checks the two match.
  const manifest = {
    name,
    version: "0.0.0",
    description: `A placeholder with no code, which created ${name} on npm: install a later version.`,
    keywords,
    homepage: "https://compmd.dev/",
    repository: { type: "git", url: `git+https://github.com/${REPOSITORY}.git`, directory },
    license: "MIT",
    files: ["README.md", "LICENSE"],
  };
  const readme = `# ${name}

A placeholder with no code. It created the package on npm, so that its versions can be published from [its folder in the CompMD repository](${folder}), by trusted publishing, with their provenance.

Install a later version: \`npm install ${name}@latest\`.
`;
  return { "package.json": `${JSON.stringify(manifest, null, 2)}\n`, "README.md": readme, LICENSE: license };
}
