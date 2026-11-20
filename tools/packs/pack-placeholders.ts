// `cargo make pack-placeholders <pack>...` (docs/releases.md, decision 8): creates new packs on npm. For each named
// pack npm doesn't have, it publishes the placeholder (`placeholder.ts`), then sets its trusted publisher, the
// repository and `release-packs.yml`, staging only. With `--dry-run` it says what it would do, and does nothing.
//
//   node tools/packs/pack-placeholders.ts [--dry-run] <pack>...

import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { npmCommand, runNpm, type RunNpm } from "./npm.ts";
import { isPackName, packageOf, placeholderFiles, REPOSITORY, WORKFLOW } from "./placeholder.ts";
import { versionsOf } from "./registry.ts";

/** npm asks for a pause between packages set up one after the other. */
const PAUSE_MS = 2000;

/** What `placeholders` does through: each a stub in its tests. */
export interface Steps {
  /** Whether npm has the package. */
  has(name: string): Promise<boolean>;
  /** Writes `files` to a new folder outside the repository, and gives its path. */
  write(files: Record<string, string>): string;
  npm: RunNpm;
  pause(): Promise<void>;
  log(line: string): void;
}

/** npm's arguments that publish a placeholder written to `folder`. */
export const publishArgs = (folder: string): string[] => ["publish", folder, "--access", "public"];

/** npm's arguments that set `name`'s trusted publisher: this repository's `release-packs.yml`, staging only. */
export const trustArgs = (name: string): string[] => ["trust", "github", name, "--file", WORKFLOW, "--repository", REPOSITORY, "--allow-stage-publish", "--yes"];

/**
 * Publishes a placeholder for each of `packs` npm doesn't have, and sets its trusted publisher.
 *
 * @returns The exit code: 0, or 2 for a name that isn't a pack's (nothing done then).
 * @throws Error when npm or the registry fails, after the packs before it are done.
 */
export async function placeholders(packs: readonly string[], license: string, options: { dryRun: boolean }, steps: Steps): Promise<number> {
  const wrong = packs.filter((pack) => !isPackName(pack));
  if (packs.length === 0 || wrong.length > 0) {
    steps.log(packs.length === 0 ? "name the packs: cargo make pack-placeholders card chart" : `not a pack's name: ${wrong.join(", ")} (lowercase words joined by "-", as embed-youtube)`);
    return 2;
  }
  const created: string[] = [];
  for (const pack of packs) {
    const name = packageOf(pack);
    if (await steps.has(name)) {
      steps.log(`${name}: npm has it already, left as it is`);
      continue;
    }
    const files = placeholderFiles(pack, license);
    if (options.dryRun) {
      steps.log(`${name}: would publish 0.0.0 (${Object.keys(files).join(", ")}), then set its trusted publisher:`);
      steps.log(`  ${npmCommand(publishArgs("<a new folder>")).join(" ")}`);
      steps.log(`  ${npmCommand(trustArgs(name)).join(" ")}`);
      continue;
    }
    if (created.length > 0) await steps.pause();
    steps.npm(publishArgs(steps.write(files)));
    await steps.pause();
    steps.npm(trustArgs(name));
    created.push(name);
    steps.log(`${name}: 0.0.0 published, its trusted publisher set (${REPOSITORY}, ${WORKFLOW}, staging only)`);
  }
  // A trusted publisher that hasn't published in 2 days expires (RELEASING.md).
  if (created.length > 0) steps.log(`Release ${created.length === 1 ? "it" : "them"} within 2 days, or the trusted publisher expires: cargo make release-packs. Deprecate each placeholder once its first version is out.`);
  return 0;
}

/** The steps as they run: the registry over HTTP, npm in a terminal of its own, folders in the system's temporary one. */
const realSteps: Steps = {
  has: async (name) => (await versionsOf(name)).length > 0,
  write(files) {
    const folder = mkdtempSync(join(tmpdir(), "compmd-placeholder-"));
    for (const [file, text] of Object.entries(files)) writeFileSync(join(folder, file), text);
    return folder;
  },
  npm: runNpm,
  pause: () => sleep(PAUSE_MS),
  log: (line) => {
    console.log(line);
  },
};

if (import.meta.main) {
  const args = process.argv.slice(2);
  const license = readFileSync(join(import.meta.dirname, "../../LICENSE-CODE"), "utf8");
  try {
    process.exitCode = await placeholders(
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
