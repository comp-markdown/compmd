// The spec's revisions (spec §1, docs/releases.md, decision 1): a change to the spec or its companion goes under
// `### Unreleased` in the changelog, which makes the spec dirty; `cut` makes the next revision of it, `1.0-draft.N`,
// and tags it; `check` fails while the spec is dirty, which a push to `main` and a release may not be.
//
//   node tools/spec-revision/spec-revision.ts cut      # cargo make spec-revision
//   node tools/spec-revision/spec-revision.ts check    # cargo make spec-check

import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export const SPEC = "spec/compmd-spec-v1.md";
export const COMPANION = "spec/compmd-registry-v1.md";
export const CHANGELOG = "spec/CHANGELOG.md";
export const UNRELEASED = "### Unreleased";

/** The spec's header: `**Version 1.0-draft.13. Last changed 2026-11-17.**`, or a release's, `**Version 1.0. …**`. */
const HEADER = /^\*\*Version (\d+)\.(\d+)(?:-draft\.(\d+))?\. Last changed (\d{4}-\d{2}-\d{2})\.\*\*/m;
/** The companion's: `**Version 1.0-draft.13, with the spec. Last changed 2026-11-02, in 1.0-draft.9.**`. */
const COMPANION_HEADER = /^\*\*Version (\S+), with the spec\. Last changed (\d{4}-\d{2}-\d{2}), in (\S+)\.\*\*$/m;

/** The spec and its companion, and the changelog, as text. */
export interface Files {
  spec: string;
  companion: string;
  changelog: string;
}

/** What `check` asks of git. */
export interface Git {
  /** Whether the tag exists. */
  hasTag(tag: string): boolean;
  /** Which of `paths` differ, in the working tree, from what they are at `tag`. */
  changedSince(tag: string, paths: readonly string[]): string[];
}

/**
 * The revision the spec's header names: `1.0-draft.13`, or a release's `1.0`; `undefined` when it names none.
 *
 * @example
 * revisionOf("**Version 1.0-draft.13. Last changed 2026-11-17.**"); // "1.0-draft.13"
 */
export function revisionOf(spec: string): string | undefined {
  const found = HEADER.exec(spec);
  if (!found) return undefined;
  const [, major, minor, draft] = found;
  return draft === undefined ? `${major}.${minor}` : `${major}.${minor}-draft.${draft}`;
}

/** The revision after `revision`: the next draft, or after a release the first draft of the next minor version. */
export function nextRevision(revision: string): string {
  const draft = /^(\d+\.\d+)-draft\.(\d+)$/.exec(revision);
  if (draft) return `${draft[1]}-draft.${String(Number(draft[2]) + 1)}`;
  const [major = "1", minor = "0"] = revision.split(".");
  return `${major}.${String(Number(minor) + 1)}-draft.1`;
}

/** A revision's tag: `spec-v1.0-draft.13`. */
export const tagOf = (revision: string): string => `spec-v${revision}`;

/** The changelog's entries under `### Unreleased`, up to the next heading; `undefined` without the heading. */
export function unreleasedOf(changelog: string): string | undefined {
  const start = changelog.indexOf(`\n${UNRELEASED}\n`);
  if (start === -1) return undefined;
  const after = changelog.slice(start + UNRELEASED.length + 2);
  const end = after.search(/^#{2,3} /m);
  return (end === -1 ? after : after.slice(0, end)).trim();
}

/**
 * The files with the next revision cut on `date` (`YYYY-MM-DD`): the changelog's `### Unreleased` becomes the
 * revision's heading, the spec's header names it with the date, and the companion's names it too, with its own date
 * and revision when `companionChanged`.
 *
 * @throws Error when the changelog has nothing unreleased, or a header isn't as expected.
 */
export function cut(files: Files, date: string, companionChanged: boolean): Files & { revision: string } {
  if (!unreleasedOf(files.changelog)) throw new Error(`${CHANGELOG} has nothing under "${UNRELEASED}": write the changes there first`);
  const current = revisionOf(files.spec);
  if (current === undefined) throw new Error(`${SPEC}'s header names no revision`);
  if (!COMPANION_HEADER.test(files.companion)) throw new Error(`${COMPANION}'s header isn't "**Version …, with the spec. Last changed …, in ….**"`);
  const revision = nextRevision(current);
  const companion = files.companion.replace(COMPANION_HEADER, (_, _version: string, changed: string, changedIn: string) =>
    companionChanged ? `**Version ${revision}, with the spec. Last changed ${date}, in ${revision}.**` : `**Version ${revision}, with the spec. Last changed ${changed}, in ${changedIn}.**`,
  );
  return {
    revision,
    spec: files.spec.replace(HEADER, `**Version ${revision}. Last changed ${date}.**`),
    companion,
    changelog: files.changelog.replace(`\n${UNRELEASED}\n`, `\n### ${revision}, ${date}\n`),
  };
}

/**
 * What keeps the spec from being pushed or released as it is: no revision in its header, no tag for it, the spec or
 * its companion changed since that tag, or entries left under `### Unreleased`. Empty when it is clean.
 */
export function problems(files: Files, git: Git): string[] {
  const revision = revisionOf(files.spec);
  if (revision === undefined) return [`${SPEC}'s header names no revision`];
  const found: string[] = [];
  const tag = tagOf(revision);
  if (!git.hasTag(tag)) found.push(`the spec's revision ${revision} has no tag ${tag}`);
  else {
    const changed = git.changedSince(tag, [SPEC, COMPANION]);
    if (changed.length > 0) found.push(`${changed.join(" and ")} changed since ${tag}`);
  }
  if (unreleasedOf(files.changelog)) found.push(`${CHANGELOG} has entries under "${UNRELEASED}"`);
  return found;
}

/** The repository's git, at `root`. */
function gitAt(root: string): Git & { commit(message: string, paths: string[]): void; tag(name: string): void } {
  const git = (...args: string[]): string => execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
  return {
    hasTag(tag) {
      try {
        git("rev-parse", "--verify", "--quiet", `refs/tags/${tag}`);
        return true;
      } catch {
        return false;
      }
    },
    changedSince: (tag, paths) => git("diff", "--name-only", tag, "--", ...paths).split("\n").filter((line) => line !== ""),
    commit: (message, paths) => void git("commit", "--message", message, "--", ...paths),
    // Annotated, so `git push --follow-tags` pushes it with the commit.
    tag: (name) => void git("tag", "--annotate", name, "--message", name),
  };
}

/** Runs `command`, `cut` or `check`, in the repository at `root`; the exit code. */
export function main(command: string | undefined, root: string, today: string): number {
  const read = (path: string) => readFileSync(join(root, path), "utf8");
  const files: Files = { spec: read(SPEC), companion: read(COMPANION), changelog: read(CHANGELOG) };
  const git = gitAt(root);
  if (command === "check") {
    const found = problems(files, git);
    for (const problem of found) console.error(`${problem}: cut a revision with \`cargo make spec-revision\` (docs/releases.md, decision 1)`);
    return found.length === 0 ? 0 : 1;
  }
  if (command !== "cut") {
    console.error("usage: node tools/spec-revision/spec-revision.ts cut|check");
    return 2;
  }
  const current = revisionOf(files.spec);
  const companionChanged = current !== undefined && git.hasTag(tagOf(current)) && git.changedSince(tagOf(current), [COMPANION]).length > 0;
  const next = cut(files, today, companionChanged);
  writeFileSync(join(root, SPEC), next.spec);
  writeFileSync(join(root, COMPANION), next.companion);
  writeFileSync(join(root, CHANGELOG), next.changelog);
  git.commit(`spec: ${next.revision}`, [SPEC, COMPANION, CHANGELOG]);
  git.tag(tagOf(next.revision));
  console.log(`cut ${next.revision}, tagged ${tagOf(next.revision)}; push it with \`git push --follow-tags\``);
  return 0;
}

if (import.meta.main) process.exit(main(process.argv[2], join(import.meta.dirname, "../.."), new Date().toISOString().slice(0, 10)));
