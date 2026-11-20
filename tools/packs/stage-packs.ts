// What `release-packs.yml` stages (docs/releases.md, decision 8): every pack tagged `<pack>-v<version>` at the
// commit it runs on whose version npm doesn't have, packed into a folder for `npm stage publish`. A pack without its
// tag at this commit isn't staged, whatever versions.json says: the tags are what `cargo make release-packs` released.
//
//   node tools/packs/stage-packs.ts plan              # the packs it would stage; fails when there are none
//   node tools/packs/stage-packs.ts pack <folder>     # packs them into the folder

import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { versionsOf } from "./registry.ts";
import { unreleased, type Release } from "./unreleased.ts";
import { differing, errors, PACKS, readPacks, type Pack, type Versions } from "./versions.ts";

/** What `toStage` does through: each a stub in its tests. */
export interface Steps {
  read(): { versions: Versions; packs: Pack[] };
  has(name: string, version: string): Promise<boolean>;
  /** The tags at the commit the workflow runs on. */
  tagsHere(): string[];
  log(line: string): void;
}

/**
 * The releases to stage at this commit.
 *
 * @throws Error when versions.json and the packs disagree, or there is nothing to stage.
 */
export async function toStage(steps: Steps): Promise<Release[]> {
  const { versions, packs } = steps.read();
  const wrong = [...errors(versions, packs), ...differing(versions, packs).map(({ pack, version }) => `${PACKS}/${pack.name} isn't at ${version}`)];
  if (wrong.length > 0) throw new Error(`versions.json and the packs disagree: ${wrong.join("; ")}`);
  const here = new Set(steps.tagsHere());
  const releases = (await unreleased(versions, steps.has)).filter((release) => here.has(release.tag));
  if (releases.length === 0) throw new Error("nothing to stage: no pack is tagged at this commit with a version npm doesn't have");
  for (const release of releases) steps.log(`${release.name} ${release.version} (${release.tag})`);
  return releases;
}

/** The steps as they run in the workflow, at `root`. */
function realSteps(root: string): Steps {
  return {
    read: () => readPacks(root),
    has: async (name, version) => (await versionsOf(name)).includes(version),
    tagsHere: () =>
      execFileSync("git", ["tag", "--points-at", "HEAD"], { cwd: root, encoding: "utf8" })
        .split("\n")
        .filter((tag) => tag !== ""),
    log: (line) => {
      console.log(line);
    },
  };
}

/** Runs `plan` or `pack <folder>`; the exit code. */
export async function main(args: readonly string[], root: string): Promise<number> {
  const [command, folder] = args;
  if (command !== "plan" && !(command === "pack" && folder)) {
    console.error("usage: node tools/packs/stage-packs.ts plan | pack <folder>");
    return 2;
  }
  const releases = await toStage(realSteps(root));
  if (command === "pack" && folder) {
    for (const release of releases) execFileSync("pnpm", ["--dir", join(PACKS, release.pack), "pack", "--pack-destination", folder], { cwd: root, stdio: "inherit" });
  }
  return 0;
}

if (import.meta.main) process.exitCode = await main(process.argv.slice(2), join(import.meta.dirname, "../.."));
