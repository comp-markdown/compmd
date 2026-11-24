import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { CHANGELOG, COMPANION, cut, cutRevision, localDay, nextRevision, problems, releaseOf, revisionOf, SPEC, unreleasedOf, type Files, type Git, type Repository } from "./spec-revision.ts";

const FILES: Files = {
  spec: "# CompMD Specification v1 (draft)\n\n**Version 1.0-draft.13. Last changed 2026-11-17.** Every change is in the changelog.\n",
  companion: "# CompMD packs on a registry\n\n**Version 1.0-draft.13, with the spec. Last changed 2026-11-02, in 1.0-draft.9.**\n",
  changelog: "# Changelog\n\n## Version 1 (draft)\n\n### Unreleased\n\n- **A change (§2).** Said.\n\n### 1.0-draft.13, 2026-11-17\n\n- Older.\n",
};

/** A git with `tags`, where `changed` paths differ from every tag. */
const gitWith = (tags: string[], changed: string[] = []): Git => ({ hasTag: (tag) => tags.includes(tag), changedSince: (_, paths) => paths.filter((path) => changed.includes(path)) });

describe("revisionOf and nextRevision", () => {
  it("read a draft's and a release's header, and number the next draft", () => {
    assert.equal(revisionOf(FILES.spec), "1.0-draft.13");
    assert.equal(revisionOf("**Version 1.0. Last changed 2027-01-04.**"), "1.0");
    assert.equal(revisionOf("**Version 1, draft. Last changed 2026-11-03.**"), undefined);
    assert.equal(nextRevision("1.0-draft.13"), "1.0-draft.14");
    assert.equal(nextRevision("1.0"), "1.1-draft.1");
  });

  it("give the release a draft leads to, and refuse a release", () => {
    assert.equal(releaseOf("1.0-draft.13"), "1.0");
    assert.throws(() => releaseOf("1.0"), /a release already/);
  });
});

describe("localDay", () => {
  it("gives the day of this machine's calendar, late in the evening and early in the morning", () => {
    assert.equal(localDay(new Date(2026, 10, 23, 23, 30)), "2026-11-23");
    assert.equal(localDay(new Date(2026, 0, 1, 0, 15)), "2026-01-01");
  });
});

describe("unreleasedOf", () => {
  it("gives the entries under Unreleased, empty when there are none, and nothing without the heading", () => {
    assert.equal(unreleasedOf(FILES.changelog), "- **A change (§2).** Said.");
    assert.equal(unreleasedOf("## V\n\n### Unreleased\n\n### 1.0-draft.13, 2026-11-17\n"), "");
    assert.equal(unreleasedOf("## V\n\n### 1.0-draft.13, 2026-11-17\n"), undefined);
  });

  it("reads a changelog with CRLF line endings, and a cut keeps them", () => {
    const crlf = FILES.changelog.replaceAll("\n", "\r\n");
    assert.equal(unreleasedOf(crlf), "- **A change (§2).** Said.");
    assert.match(cut({ ...FILES, changelog: crlf }, "2026-11-20", false).changelog, /\r\n### 1\.0-draft\.14, 2026-11-20\r\n/);
  });
});

describe("cut", () => {
  it("makes Unreleased the next revision, in the changelog, the spec's header and the companion's", () => {
    const next = cut(FILES, "2026-11-20", false);
    assert.equal(next.revision, "1.0-draft.14");
    assert.match(next.changelog, /\n### 1\.0-draft\.14, 2026-11-20\n\n- \*\*A change/);
    assert.doesNotMatch(next.changelog, /Unreleased/);
    assert.match(next.spec, /^\*\*Version 1\.0-draft\.14\. Last changed 2026-11-20\.\*\* Every change/m);
    assert.match(next.companion, /^\*\*Version 1\.0-draft\.14, with the spec\. Last changed 2026-11-02, in 1\.0-draft\.9\.\*\*$/m);
    assert.match(cut(FILES, "2026-11-20", true).companion, /Last changed 2026-11-20, in 1\.0-draft\.14\./);
  });

  it("cuts a release's revision, 1.0, from a draft", () => {
    const release = cut(FILES, "2027-01-04", true, true);
    assert.equal(release.revision, "1.0");
    assert.match(release.spec, /^\*\*Version 1\.0\. Last changed 2027-01-04\.\*\*/m);
    assert.equal(revisionOf(release.spec), "1.0");
    assert.match(release.changelog, /\n### 1\.0, 2027-01-04\n/);
    assert.match(release.companion, /^\*\*Version 1\.0, with the spec\. Last changed 2027-01-04, in 1\.0\.\*\*$/m);
  });

  it("refuses a changelog with nothing unreleased, and headers it can't read", () => {
    assert.throws(() => cut({ ...FILES, changelog: "## V\n\n### Unreleased\n\n" }, "2026-11-20", false), /nothing under "### Unreleased"/);
    assert.throws(() => cut({ ...FILES, spec: "no header" }, "2026-11-20", false), /names no revision/);
    assert.throws(() => cut({ ...FILES, companion: "no header" }, "2026-11-20", false), /compmd-registry-v1\.md's header/);
  });
});

describe("cutRevision", () => {
  /** A repository folder holding `FILES`; and a git with `tags`, recording what it was asked, whose commit `fails` if given. */
  const repository = (tags: string[], fails?: string) => {
    const root = mkdtempSync(join(tmpdir(), "spec-revision-"));
    mkdirSync(join(root, "spec"));
    writeFileSync(join(root, SPEC), FILES.spec);
    writeFileSync(join(root, COMPANION), FILES.companion);
    writeFileSync(join(root, CHANGELOG), FILES.changelog);
    const asked: string[] = [];
    const git: Repository = {
      ...gitWith(tags),
      commit: (message) => {
        asked.push(`commit ${message}`);
        if (fails !== undefined) throw new Error(fails);
      },
      tag: (name) => {
        asked.push(`tag ${name}`);
      },
    };
    const files = () => [SPEC, COMPANION, CHANGELOG].map((path) => readFileSync(join(root, path), "utf8"));
    return { root, git, asked, files };
  };

  it("writes the next revision, commits it and tags it", () => {
    const { root, git, asked, files } = repository(["spec-v1.0-draft.13"]);
    assert.equal(cutRevision(root, "2026-11-20", git), "1.0-draft.14");
    assert.deepEqual(asked, ["commit spec: 1.0-draft.14", "tag spec-v1.0-draft.14"]);
    assert.equal(revisionOf(files()[0] ?? ""), "1.0-draft.14");
  });

  it("leaves the three files as they were when the commit fails", () => {
    const { root, git, files } = repository(["spec-v1.0-draft.13"], "a hook refused it");
    assert.throws(() => cutRevision(root, "2026-11-20", git), /the commit failed, and the three files are as they were: a hook refused it/);
    assert.deepEqual(files(), [FILES.spec, FILES.companion, FILES.changelog]);
  });

  it("stops before anything is written when the next revision's tag exists, or the current one's is missing", () => {
    const tagged = repository(["spec-v1.0-draft.13", "spec-v1.0-draft.14"]);
    assert.throws(() => cutRevision(tagged.root, "2026-11-20", tagged.git), /spec-v1\.0-draft\.14 exists already: nothing was written/);
    assert.deepEqual([tagged.files(), tagged.asked], [[FILES.spec, FILES.companion, FILES.changelog], []]);
    const untagged = repository([]);
    assert.throws(() => cutRevision(untagged.root, "2026-11-20", untagged.git), /1\.0-draft\.13 has no tag spec-v1\.0-draft\.13, so whether spec\/compmd-registry-v1\.md changed since it can't be told/);
    assert.deepEqual(untagged.files(), [FILES.spec, FILES.companion, FILES.changelog]);
  });
});

describe("problems", () => {
  const clean: Files = { ...FILES, changelog: "## V\n\n### 1.0-draft.13, 2026-11-17\n\n- Older.\n" };

  it("finds none in a spec its tag holds, with nothing unreleased", () => {
    assert.deepEqual(problems(clean, gitWith(["spec-v1.0-draft.13"])), []);
  });

  it("finds a revision without its tag, a spec changed since it, and unreleased entries", () => {
    assert.deepEqual(problems(clean, gitWith([])), ["the spec's revision 1.0-draft.13 has no tag spec-v1.0-draft.13"]);
    assert.deepEqual(problems(clean, gitWith(["spec-v1.0-draft.13"], ["spec/compmd-spec-v1.md"])), ["spec/compmd-spec-v1.md changed since spec-v1.0-draft.13"]);
    assert.deepEqual(problems(FILES, gitWith(["spec-v1.0-draft.13"])), ['spec/CHANGELOG.md has entries under "### Unreleased"']);
    assert.deepEqual(problems({ ...clean, spec: "no header" }, gitWith([])), ["spec/compmd-spec-v1.md's header names no revision"]);
  });
});
