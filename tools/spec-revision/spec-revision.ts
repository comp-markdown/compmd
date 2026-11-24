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

/**
 * The release a draft leads to: `1.0` for `1.0-draft.14`.
 *
 * @throws Error when `revision` is a release already.
 */
export function releaseOf(revision: string): string {
  const draft = /^(\d+\.\d+)-draft\.\d+$/.exec(revision);
  if (!draft?.[1]) throw new Error(`the spec is ${revision}, a release already: a release is cut from a draft`);
  return draft[1];
}

/** A revision's tag: `spec-v1.0-draft.13`. */
export const tagOf = (revision: string): string => `spec-v${revision}`;

/** The `### Unreleased` heading's line, with its line ending, LF or CRLF. */
const UNRELEASED_LINE = /^### Unreleased(\r?)$/m;

/** The changelog's entries under `### Unreleased`, up to the next heading; `undefined` without the heading. */
export function unreleasedOf(changelog: string): string | undefined {
  const found = UNRELEASED_LINE.exec(changelog);
  if (!found) return undefined;
  const after = changelog.slice(found.index + found[0].length);
  const end = after.search(/^#{2,3} /m);
  return (end === -1 ? after : after.slice(0, end)).trim();
}

/**
 * The files with the next revision cut on `date` (`YYYY-MM-DD`): the changelog's `### Unreleased` becomes the
 * revision's heading, the spec's header names it with the date, and the companion's names it too, with its own date
 * and revision when `companionChanged`. The next revision is the next draft, or with `release` the release the draft
 * leads to (`1.0`).
 *
 * @throws Error when the changelog has nothing unreleased, a header isn't as expected, or `release` and the spec is
 * a release already.
 */
export function cut(files: Files, date: string, companionChanged: boolean, release = false): Files & { revision: string } {
  if (!unreleasedOf(files.changelog)) throw new Error(`${CHANGELOG} has nothing under "${UNRELEASED}": write the changes there first`);
  const current = revisionOf(files.spec);
  if (current === undefined) throw new Error(`${SPEC}'s header names no revision`);
  if (!COMPANION_HEADER.test(files.companion)) throw new Error(`${COMPANION}'s header isn't "**Version …, with the spec. Last changed …, in ….**"`);
  const revision = release ? releaseOf(current) : nextRevision(current);
  const companion = files.companion.replace(COMPANION_HEADER, (_, _version: string, changed: string, changedIn: string) =>
    companionChanged ? `**Version ${revision}, with the spec. Last changed ${date}, in ${revision}.**` : `**Version ${revision}, with the spec. Last changed ${changed}, in ${changedIn}.**`,
  );
  return {
    revision,
    spec: files.spec.replace(HEADER, `**Version ${revision}. Last changed ${date}.**`),
    companion,
    changelog: files.changelog.replace(UNRELEASED_LINE, (_, ending: string) => `### ${revision}, ${date}${ending}`),
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

/** What `cut` asks of git besides `check`'s: a commit of `paths`, and a tag on it. */
export interface Repository extends Git {
  commit(message: string, paths: string[]): void;
  tag(name: string): void;
}

/**
 * Cuts the next revision of the spec in `root` on `today`, as {@link cut} does, writes it, commits it and tags it,
 * whole or not at all (audit B26): the tag is checked before anything is written, and a commit that fails puts the
 * three files back as they were. The revision cut.
 *
 * @throws Error when the current revision has no tag (so whether the companion changed since it can't be told), the
 * next revision's tag exists, the cut can't be made, or the commit fails; or when the tag fails after the commit,
 * saying how to tag it.
 */
export function cutRevision(root: string, today: string, git: Repository, { release = false } = {}): string {
  const paths = [SPEC, COMPANION, CHANGELOG] as const;
  const [spec, companion, changelog] = paths.map((path) => readFileSync(join(root, path), "utf8")) as [string, string, string];
  const current = revisionOf(spec);
  if (current !== undefined && !git.hasTag(tagOf(current))) {
    throw new Error(`the spec's revision ${current} has no tag ${tagOf(current)}, so whether ${COMPANION} changed since it can't be told: tag the commit that cut it first`);
  }
  const companionChanged = current !== undefined && git.changedSince(tagOf(current), [COMPANION]).length > 0;
  const next = cut({ spec, companion, changelog }, today, companionChanged, release);
  const tag = tagOf(next.revision);
  if (git.hasTag(tag)) throw new Error(`${tag} exists already: nothing was written`);
  const write = (texts: readonly string[]) => {
    paths.forEach((path, index) => {
      writeFileSync(join(root, path), texts[index] ?? "");
    });
  };
  write([next.spec, next.companion, next.changelog]);
  try {
    git.commit(`spec: ${next.revision}`, [...paths]);
  } catch (error) {
    write([spec, companion, changelog]);
    throw new Error(`the commit failed, and the three files are as they were: ${error instanceof Error ? error.message : String(error)}`);
  }
  try {
    git.tag(tag);
  } catch (error) {
    throw new Error(`committed ${next.revision}, but tagging it failed: ${error instanceof Error ? error.message : String(error)}; tag it with \`git tag --annotate ${tag} --message ${tag}\``);
  }
  return next.revision;
}

/** The repository's git, at `root`. */
function gitAt(root: string): Repository {
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

/** Today as this machine's calendar has it, `YYYY-MM-DD`: not UTC's, which is another day near midnight. */
export function localDay(date: Date = new Date()): string {
  const two = (n: number): string => String(n).padStart(2, "0");
  return `${String(date.getFullYear())}-${two(date.getMonth() + 1)}-${two(date.getDate())}`;
}

const USAGE = "usage: node tools/spec-revision/spec-revision.ts cut [--release] | check";

/** Runs `args`, `cut` (with `--release` for a release's revision) or `check`, in the repository at `root`; the exit code. */
export function main(args: readonly string[], root: string, today: string, git: Repository = gitAt(root)): number {
  const [command, ...rest] = args;
  if (command === "check") {
    const read = (path: string) => readFileSync(join(root, path), "utf8");
    const found = problems({ spec: read(SPEC), companion: read(COMPANION), changelog: read(CHANGELOG) }, git);
    for (const problem of found) console.error(`${problem}: cut a revision with \`cargo make spec-revision\` (docs/releases.md, decision 1)`);
    return found.length === 0 ? 0 : 1;
  }
  const release = rest.length === 1 && rest[0] === "--release";
  if (command !== "cut" || (rest.length > 0 && !release)) {
    console.error(USAGE);
    return 2;
  }
  try {
    const revision = cutRevision(root, today, git, { release });
    console.log(`cut ${revision}, tagged ${tagOf(revision)}; push it with \`git push --follow-tags\``);
    return 0;
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    return 1;
  }
}

if (import.meta.main) process.exit(main(process.argv.slice(2), join(import.meta.dirname, "../.."), localDay()));
