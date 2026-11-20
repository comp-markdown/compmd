// `cargo make release-packs` (docs/releases.md, decision 8): releases every pack whose version in
// `compmd-packs/versions.json` npm doesn't have. It writes the versions into the packs' files when they differ (then
// stops, for a commit), shows the releases, and on the author's yes tags each `<pack>-v<version>`, pushes the tags and
// starts one run of `release-packs.yml`, which stages them all. With `--dry-run` it says what it would do, and does
// nothing.
//
//   node tools/packs/release-packs.ts [--dry-run]

import { execFileSync } from "node:child_process";
import { join } from "node:path";
import { createInterface } from "node:readline/promises";
import { versionsOf } from "./registry.ts";
import { unreleased, type Release } from "./unreleased.ts";
import { differing, errors, PACKS, readPacks, VERSIONS, withVersion, writePack, type Pack, type Versions } from "./versions.ts";

/** The workflow that stages the releases. */
export const WORKFLOW = "release-packs.yml";
/** The branch it runs on, whose commit the tags must be. */
export const BRANCH = "main";

/** What `releasePacks` does through: each a stub in its tests. */
export interface Steps {
  read(): { versions: Versions; packs: Pack[] };
  write(pack: Pack): void;
  has(name: string, version: string): Promise<boolean>;
  git: {
    /** The commit of a revision, `HEAD` or `origin/main`. */
    commitOf(revision: string): string;
    /** Whether the packs' folder has changes not committed. */
    uncommitted(): boolean;
    /** The commit a tag names, or `undefined` without the tag. */
    tagged(tag: string): string | undefined;
    tag(name: string): void;
    push(tags: readonly string[]): void;
  };
  runWorkflow(): void;
  confirm(question: string): Promise<boolean>;
  log(line: string): void;
}

const describe = (release: Release) => `${release.name} ${release.version} (${release.tag})`;

/**
 * Releases the packs whose version npm doesn't have.
 *
 * @returns The exit code: 0 when released, nothing to release, or a dry run; 1 when something must be done first
 *   (said in the log), or the author said no.
 */
export async function releasePacks(options: { dryRun: boolean }, steps: Steps): Promise<number> {
  const { versions, packs } = steps.read();
  const wrong = errors(versions, packs);
  if (wrong.length > 0) {
    for (const problem of wrong) steps.log(problem);
    return 1;
  }
  const behind = differing(versions, packs);
  if (behind.length > 0) {
    for (const { pack, version, written } of behind) steps.log(`${PACKS}/${pack.name}: ${written.join(" and ")} → ${version}`);
    if (options.dryRun) steps.log(`would write these into the packs' package.json and index.comp.md, to be committed before the release`);
    else {
      for (const { pack, version } of behind) steps.write(withVersion(pack, version));
      steps.log(`wrote ${VERSIONS}'s versions into the packs' package.json and index.comp.md: commit them, then run this again`);
      return 1;
    }
  }
  const releases = await unreleased(versions, steps.has);
  if (releases.length === 0) {
    steps.log("npm has every pack's version in versions.json: nothing to release");
    return 0;
  }
  steps.log(`to release, ${String(releases.length)}:`);
  for (const release of releases) steps.log(`  ${describe(release)}`);
  if (options.dryRun) {
    steps.log(`would tag each at ${BRANCH}'s commit, push the tags, and run ${WORKFLOW} once, which stages them on npm for you to approve`);
    return 0;
  }
  const head = steps.git.commitOf("HEAD");
  if (head !== steps.git.commitOf(`origin/${BRANCH}`) || steps.git.uncommitted()) {
    steps.log(`the release is ${BRANCH}'s commit as GitHub has it: commit and push everything first, then run this again`);
    return 1;
  }
  const elsewhere = releases.filter((release) => {
    const at = steps.git.tagged(release.tag);
    return at !== undefined && at !== head;
  });
  if (elsewhere.length > 0) {
    steps.log(`already tagged at another commit: ${elsewhere.map((release) => release.tag).join(", ")}. Give the pack a new version in ${VERSIONS}`);
    return 1;
  }
  if (!(await steps.confirm(`Release ${releases.length === 1 ? "it" : `these ${String(releases.length)}`}? [y/N] `))) return 1;
  for (const release of releases) if (steps.git.tagged(release.tag) === undefined) steps.git.tag(release.tag);
  steps.git.push(releases.map((release) => release.tag));
  steps.runWorkflow();
  steps.log(`tagged and pushed; ${WORKFLOW} runs \`cargo make ci\` once, then stages them: approve them with \`cargo make approve-packs\``);
  return 0;
}

/** The steps as they run: the repository at `root`, git, gh, the registry over HTTP, and the terminal. */
function realSteps(root: string): Steps {
  const git = (...args: string[]) => execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  return {
    read: () => readPacks(root),
    write: (pack) => {
      writePack(root, pack);
    },
    has: async (name, version) => (await versionsOf(name)).includes(version),
    git: {
      commitOf: (revision) => git("rev-parse", `${revision}^{commit}`),
      uncommitted: () => git("status", "--porcelain", "--", PACKS) !== "",
      tagged(tag) {
        try {
          return git("rev-parse", "--verify", "--quiet", `refs/tags/${tag}^{commit}`);
        } catch {
          return undefined;
        }
      },
      // Annotated, as the spec's revisions are.
      tag: (name) => void git("tag", "--annotate", name, "--message", name),
      push: (tags) => void git("push", "origin", ...tags.map((tag) => `refs/tags/${tag}`)),
    },
    runWorkflow: () => void execFileSync("gh", ["workflow", "run", WORKFLOW, "--ref", BRANCH], { cwd: root, stdio: "inherit" }),
    async confirm(question) {
      const terminal = createInterface({ input: process.stdin, output: process.stdout });
      const answer = await terminal.question(question);
      terminal.close();
      return /^y(es)?$/i.test(answer.trim());
    },
    log: (line) => {
      console.log(line);
    },
  };
}

if (import.meta.main) process.exitCode = await releasePacks({ dryRun: process.argv.includes("--dry-run") }, realSteps(join(import.meta.dirname, "../..")));
