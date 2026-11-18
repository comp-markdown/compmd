import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { cut, nextRevision, problems, revisionOf, unreleasedOf, type Files, type Git } from "./spec-revision.ts";

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
});

describe("unreleasedOf", () => {
  it("gives the entries under Unreleased, empty when there are none, and nothing without the heading", () => {
    assert.equal(unreleasedOf(FILES.changelog), "- **A change (§2).** Said.");
    assert.equal(unreleasedOf("## V\n\n### Unreleased\n\n### 1.0-draft.13, 2026-11-17\n"), "");
    assert.equal(unreleasedOf("## V\n\n### 1.0-draft.13, 2026-11-17\n"), undefined);
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

  it("refuses a changelog with nothing unreleased, and headers it can't read", () => {
    assert.throws(() => cut({ ...FILES, changelog: "## V\n\n### Unreleased\n\n" }, "2026-11-20", false), /nothing under "### Unreleased"/);
    assert.throws(() => cut({ ...FILES, spec: "no header" }, "2026-11-20", false), /names no revision/);
    assert.throws(() => cut({ ...FILES, companion: "no header" }, "2026-11-20", false), /compmd-registry-v1\.md's header/);
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
