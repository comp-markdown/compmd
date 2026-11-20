import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { releasePacks, type Steps } from "./release-packs.ts";
import { packAt } from "./testing.ts";
import type { Pack, Versions } from "./versions.ts";

/** Steps over `versions` and `packs`, npm having `published` (`name@version`), recording what they were asked. */
function stubs(versions: Versions, packs: Pack[], published: readonly string[], options: { answer?: boolean; head?: string; main?: string; uncommitted?: boolean; tags?: Record<string, string> } = {}) {
  const done: string[] = [];
  const log: string[] = [];
  const tags = { ...options.tags };
  const steps: Steps = {
    read: () => ({ versions, packs }),
    write: (pack) => {
      done.push(`write ${pack.name}`);
    },
    has: (name, version) => Promise.resolve(published.includes(`${name}@${version}`)),
    git: {
      commitOf: (revision) => (revision === "HEAD" ? (options.head ?? "abc") : (options.main ?? "abc")),
      uncommitted: () => options.uncommitted ?? false,
      tagged: (tag) => tags[tag],
      tag: (name) => {
        tags[name] = options.head ?? "abc";
        done.push(`tag ${name}`);
      },
      push: (pushed) => {
        done.push(`push ${pushed.join(" ")}`);
      },
    },
    runWorkflow: () => {
      done.push("run release-packs.yml");
    },
    confirm: () => {
      done.push("asked");
      return Promise.resolve(options.answer ?? true);
    },
    log: (line) => {
      log.push(line);
    },
  };
  return { steps, done, log };
}

const PACKS = [packAt("card", "0.2.0"), packAt("mermaid", "0.1.2"), packAt("wireframe", "0.1.0")];
const VERSIONS = { card: "0.2.0", mermaid: "0.1.2", wireframe: "0.1.0" };
const PUBLISHED = ["@comp-markdown/card@0.1.0", "@comp-markdown/mermaid@0.1.2", "@comp-markdown/wireframe@0.1.0"];

describe("cargo make release-packs", () => {
  it("tags each pack whose version npm doesn't have, pushes the tags, and runs the workflow once", async () => {
    const { steps, done, log } = stubs(VERSIONS, PACKS, PUBLISHED);
    assert.equal(await releasePacks({ dryRun: false }, steps), 0);
    assert.deepEqual(done, ["asked", "tag card-v0.2.0", "push card-v0.2.0", "run release-packs.yml"]);
    assert.deepEqual(log.slice(0, 2), ["to release, 1:", "  @comp-markdown/card 0.2.0 (card-v0.2.0)"]);
  });

  it("with --dry-run, shows what it would release and does nothing", async () => {
    const { steps, done, log } = stubs({ ...VERSIONS, wireframe: "0.2.0" }, [packAt("card", "0.2.0"), packAt("mermaid", "0.1.2"), packAt("wireframe", "0.2.0")], PUBLISHED);
    assert.equal(await releasePacks({ dryRun: true }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(log, [
      "to release, 2:",
      "  @comp-markdown/card 0.2.0 (card-v0.2.0)",
      "  @comp-markdown/wireframe 0.2.0 (wireframe-v0.2.0)",
      "would tag each at main's commit, push the tags, and run release-packs.yml once, which stages them on npm for you to approve",
    ]);
  });

  it("releases nothing when npm has every version", async () => {
    const { steps, done, log } = stubs({ ...VERSIONS, card: "0.1.0" }, [packAt("card", "0.1.0"), ...PACKS.slice(1)], PUBLISHED);
    assert.equal(await releasePacks({ dryRun: false }, steps), 0);
    assert.deepEqual(done, []);
    assert.match(log.join("\n"), /nothing to release/);
  });

  it("writes versions.json's versions into the packs' files, and stops for them to be committed", async () => {
    const { steps, done, log } = stubs({ ...VERSIONS, card: "0.3.0" }, PACKS, PUBLISHED);
    assert.equal(await releasePacks({ dryRun: false }, steps), 1);
    assert.deepEqual(done, ["write card"]);
    assert.match(log.at(-1) ?? "", /commit them, then run this again/);
  });

  it("with --dry-run, says it would write them, and goes on to what it would release", async () => {
    const { steps, done, log } = stubs({ ...VERSIONS, card: "0.3.0" }, PACKS, PUBLISHED);
    assert.equal(await releasePacks({ dryRun: true }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(log.slice(0, 3), ["compmd-packs/card: 0.2.0 and 0.2.0 → 0.3.0", "would write these into the packs' package.json and index.comp.md, to be committed before the release", "to release, 1:"]);
  });

  it("does nothing while versions.json and the packs disagree on which packs there are", async () => {
    const { steps, done, log } = stubs({ card: "0.2.0" }, PACKS, PUBLISHED);
    assert.equal(await releasePacks({ dryRun: false }, steps), 1);
    assert.deepEqual(done, []);
    assert.equal(log.length, 2);
  });

  it("does nothing until main's commit is pushed and the packs committed", async () => {
    for (const options of [{ head: "abc", main: "def" }, { uncommitted: true }]) {
      const { steps, done, log } = stubs(VERSIONS, PACKS, PUBLISHED, options);
      assert.equal(await releasePacks({ dryRun: false }, steps), 1);
      assert.deepEqual(done, []);
      assert.match(log.at(-1) ?? "", /commit and push everything first/);
    }
  });

  it("refuses a tag at another commit, and keeps one already at this commit (a run that stopped halfway)", async () => {
    const other = stubs(VERSIONS, PACKS, PUBLISHED, { tags: { "card-v0.2.0": "old" } });
    assert.equal(await releasePacks({ dryRun: false }, other.steps), 1);
    assert.deepEqual(other.done, []);
    assert.match(other.log.at(-1) ?? "", /already tagged at another commit: card-v0.2.0/);
    const again = stubs(VERSIONS, PACKS, PUBLISHED, { tags: { "card-v0.2.0": "abc" } });
    assert.equal(await releasePacks({ dryRun: false }, again.steps), 0);
    assert.deepEqual(again.done, ["asked", "push card-v0.2.0", "run release-packs.yml"]);
  });

  it("does nothing when the author says no", async () => {
    const { steps, done } = stubs(VERSIONS, PACKS, PUBLISHED, { answer: false });
    assert.equal(await releasePacks({ dryRun: false }, steps), 1);
    assert.deepEqual(done, ["asked"]);
  });
});
