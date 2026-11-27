// A crate's placeholder (RELEASING.md, *A placeholder first*): a `0.0.0` with no code, which creates the crate on
// crates.io, so its trusted publisher can be set and the release workflow's `crates` job can publish its first real
// version. crates.io is asked through its sparse index, which needs no account and no user agent.

import type { Fetch } from "./registry.ts";
import { REPOSITORY } from "./placeholder.ts";

/** Where the sparse index keeps the crates' records. */
export const CRATES_INDEX = "https://index.crates.io";

/** Whether `name` is one of this repository's crates' names: `compmd`, or `compmd-` and lowercase words. */
export const isCrateName = (name: string): boolean => /^compmd(?:-[a-z0-9]+)*$/.test(name);

/**
 * The sparse index's address of the crate `name` (4 characters or more, as every `compmd` crate's is): its first two
 * characters, its next two, then the name.
 */
export const crateIndexUrl = (name: string): string => `${CRATES_INDEX}/${name.slice(0, 2)}/${name.slice(2, 4)}/${name}`;

/**
 * Whether crates.io has the crate `name`.
 *
 * @throws Error when the index answers anything but the crate or "not found".
 */
export async function hasCrate(name: string, fetch: Fetch = globalThis.fetch): Promise<boolean> {
  const response = await fetch(crateIndexUrl(name));
  if (response.status === 404) return false;
  if (!response.ok) throw new Error(`crates.io's index answered ${String(response.status)} for ${name}`);
  return true;
}

/** The crate's folder in the repository: `compmd-lsp`'s is `crates/lsp`. */
export const crateDirectory = (name: string): string => `crates/${name === "compmd" ? "processor" : name.replace(/^compmd-/, "")}`;

/**
 * The placeholder crate's files, by path: a `Cargo.toml` that says what it is, an empty `src/lib.rs`, a README pointing
 * to the real versions, and `license`, the MIT licence the crate's code is under.
 *
 * @example
 * crateFiles("compmd-lsp", mit)["Cargo.toml"]; // [package] name = "compmd-lsp", version = "0.0.0", …
 */
export function crateFiles(name: string, license: string): Record<string, string> {
  const folder = `https://github.com/${REPOSITORY}/tree/main/${crateDirectory(name)}`;
  const manifest = `[package]
name = "${name}"
version = "0.0.0"
edition = "2024"
description = "A placeholder with no code, which created ${name} on crates.io: use a later version."
license = "MIT"
repository = "https://github.com/${REPOSITORY}"
homepage = "https://compmd.dev/"
readme = "README.md"
`;
  const readme = `# ${name}

A placeholder with no code. It created the crate on crates.io, so that its versions can be published from [its folder in the CompMD repository](${folder}), by trusted publishing.

Use a later version: \`cargo add ${name}\`.
`;
  return {
    "Cargo.toml": manifest,
    "src/lib.rs": `//! A placeholder with no code: use a later version of ${name}.\n`,
    "README.md": readme,
    LICENSE: license,
  };
}
