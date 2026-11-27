// `cargo make release-placeholders <name>...` (RELEASING.md, *A placeholder first*): creates the implementation's new
// packages before the first release that publishes them. For an `@compmd/` package npm doesn't have, it publishes the
// placeholder (`placeholder.ts`), then sets its trusted publisher, the repository and `release.yml`, staging only. For a
// crate crates.io doesn't have, it publishes the placeholder crate (`crate-placeholder.ts`) with the author's `cargo
// login`, and says how to set its trusted publisher on crates.io, which is the author's step on the website. With
// `--dry-run` it says what it would do, and does nothing.
//
//   node tools/packs/release-placeholders.ts [--dry-run] <name>...

import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { crateFiles, hasCrate, isCrateName } from "./crate-placeholder.ts";
import { npmCommand, runNpm, type RunNpm } from "./npm.ts";
import { publishArgs, trustArgs } from "./pack-placeholders.ts";
import { placeholderFiles, type PlaceholderOf } from "./placeholder.ts";
import { versionsOf } from "./registry.ts";

/** The workflow that stages the implementation's releases and publishes its crates. */
export const RELEASE_WORKFLOW = "release.yml";
/** The GitHub environment the release workflow's `crates` job waits in. */
export const CRATES_ENVIRONMENT = "crates-io";

/** npm asks for a pause between packages set up one after the other. */
const PAUSE_MS = 2000;

/** Runs cargo with some arguments in a folder; throws when it fails. */
export type RunCargo = (args: readonly string[], folder: string) => void;

/** What `releasePlaceholders` does through: each a stub in its tests. */
export interface Steps {
  /** Whether npm has the package. */
  hasPackage(name: string): Promise<boolean>;
  /** Whether crates.io has the crate. */
  hasCrate(name: string): Promise<boolean>;
  /** Writes `files` (by path) to a new folder outside the repository, and gives its path. */
  write(files: Record<string, string>): string;
  npm: RunNpm;
  cargo: RunCargo;
  pause(): Promise<void>;
  log(line: string): void;
}

/** Whether `name` is one of the implementation's npm packages: `@compmd/` and lowercase words joined by `-`. */
export const isImplementationPackage = (name: string): boolean => /^@compmd\/[a-z0-9]+(?:-[a-z0-9]+)*$/.test(name);

/** What the placeholder of `name`, an `@compmd/` package, is of: the language server's platform packages are written from `npm-packages/lsp`. */
export function implementationPlaceholder(name: string): PlaceholderOf {
  const bare = name.replace(/^@compmd\//, "");
  return { name, directory: `npm-packages/${bare.startsWith("lsp-") ? "lsp" : bare}`, keywords: ["compmd"] };
}

/** cargo's arguments that publish the placeholder crate in the folder it runs in. */
export const cargoPublishArgs: readonly string[] = ["publish", "--allow-dirty"];

/** What the author does on crates.io for the crate `name`'s trusted publisher, which no command sets. */
export const crateTrustStep = (name: string): string =>
  `set its trusted publisher on https://crates.io/crates/${name}/settings, Trusted Publishing, GitHub: the owner comp-markdown, the repository compmd, the workflow ${RELEASE_WORKFLOW}, the environment ${CRATES_ENVIRONMENT}`;

/**
 * Publishes a placeholder of each of `names` its registry doesn't have: an `@compmd/` package on npm, with its trusted
 * publisher set; a crate on crates.io, its trusted publisher's steps said.
 *
 * @returns The exit code: 0, or 2 for a name that is neither (nothing done then).
 * @throws Error when npm, cargo or a registry fails, after the names before it are done.
 */
export async function releasePlaceholders(names: readonly string[], license: string, options: { dryRun: boolean }, steps: Steps): Promise<number> {
  const wrong = names.filter((name) => !isImplementationPackage(name) && !isCrateName(name));
  if (names.length === 0 || wrong.length > 0) {
    steps.log(
      names.length === 0
        ? "name the packages and crates: cargo make release-placeholders @compmd/lsp compmd-lsp"
        : `neither an @compmd/ package nor a compmd crate: ${wrong.join(", ")}`,
    );
    return 2;
  }
  const created: string[] = [];
  const crates: string[] = [];
  for (const name of names) {
    const npm = isImplementationPackage(name);
    if (npm ? await steps.hasPackage(name) : await steps.hasCrate(name)) {
      steps.log(`${name}: ${npm ? "npm" : "crates.io"} has it already, left as it is`);
      continue;
    }
    if (npm) {
      await placePackage(name, license, options, steps, created.length > 0);
    } else {
      placeCrate(name, license, options, steps);
      crates.push(name);
    }
    created.push(name);
  }
  if (options.dryRun || created.length === 0) return 0;
  for (const name of crates) steps.log(`${name}: ${crateTrustStep(name)}`);
  // A trusted publisher that hasn't published in 2 days expires (RELEASING.md).
  steps.log(`Release within 2 days, or the trusted publishers expire: the tag v<version> (RELEASING.md). Deprecate the npm placeholders and yank the crates' once the first versions are out.`);
  return 0;
}

/** The placeholder of `name` on npm, then its trusted publisher; after a pause when one came before it. */
async function placePackage(name: string, license: string, options: { dryRun: boolean }, steps: Steps, after: boolean): Promise<void> {
  const files = placeholderFiles(implementationPlaceholder(name), license);
  if (options.dryRun) {
    steps.log(`${name}: would publish 0.0.0 (${Object.keys(files).join(", ")}), then set its trusted publisher:`);
    steps.log(`  ${npmCommand(publishArgs("<a new folder>")).join(" ")}`);
    steps.log(`  ${npmCommand(trustArgs(name, RELEASE_WORKFLOW)).join(" ")}`);
    return;
  }
  if (after) await steps.pause();
  steps.npm(publishArgs(steps.write(files)));
  await steps.pause();
  steps.npm(trustArgs(name, RELEASE_WORKFLOW));
  steps.log(`${name}: 0.0.0 published, its trusted publisher set (${RELEASE_WORKFLOW}, staging only)`);
}

/** The placeholder crate `name` on crates.io. */
function placeCrate(name: string, license: string, options: { dryRun: boolean }, steps: Steps): void {
  const files = crateFiles(name, license);
  if (options.dryRun) {
    steps.log(`${name}: would publish 0.0.0 (${Object.keys(files).join(", ")}) from a new folder: cargo ${cargoPublishArgs.join(" ")}`);
    steps.log(`  then ${crateTrustStep(name)}`);
    return;
  }
  steps.cargo(cargoPublishArgs, steps.write(files));
  steps.log(`${name}: 0.0.0 published`);
}

/** The steps as they run: the registries over HTTP, npm and cargo in the terminal, folders in the system's temporary one. */
const realSteps: Steps = {
  hasPackage: async (name) => (await versionsOf(name)).length > 0,
  hasCrate: (name) => hasCrate(name),
  write(files) {
    const folder = mkdtempSync(join(tmpdir(), "compmd-placeholder-"));
    for (const [file, text] of Object.entries(files)) {
      mkdirSync(dirname(join(folder, file)), { recursive: true });
      writeFileSync(join(folder, file), text);
    }
    return folder;
  },
  npm: runNpm,
  cargo(args, folder) {
    const { status } = spawnSync("cargo", args, { cwd: folder, stdio: "inherit" });
    if (status !== 0) throw new Error(`cargo ${args.join(" ")} failed in ${folder}, exit code ${String(status)}`);
  },
  pause: () => sleep(PAUSE_MS),
  log: (line) => {
    console.log(line);
  },
};

if (import.meta.main) {
  const args = process.argv.slice(2);
  const license = readFileSync(join(import.meta.dirname, "../../LICENSE-CODE"), "utf8");
  try {
    process.exitCode = await releasePlaceholders(
      args.filter((arg) => arg !== "--dry-run"),
      license,
      { dryRun: args.includes("--dry-run") },
      realSteps,
    );
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
