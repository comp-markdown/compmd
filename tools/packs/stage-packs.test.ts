import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { toStage, type Steps } from "./stage-packs.ts";
import { packAt } from "./testing.ts";
import type { Pack, Versions } from "./versions.ts";

function stubs(versions: Versions, packs: Pack[], published: readonly string[], tags: readonly string[]) {
  const log: string[] = [];
  const steps: Steps = {
    read: () => ({ versions, packs }),
    has: (name, version) => Promise.resolve(published.includes(`${name}@${version}`)),
    tagsHere: () => [...tags],
    log: (line) => {
      log.push(line);
    },
  };
  return { steps, log };
}

const PACKS = [packAt("card", "0.2.0"), packAt("chart", "0.1.0"), packAt("mermaid", "0.1.2")];
const VERSIONS = { card: "0.2.0", chart: "0.1.0", mermaid: "0.1.2" };

describe("what release-packs.yml stages", () => {
  it("is each pack tagged at this commit whose version npm doesn't have", async () => {
    const { steps, log } = stubs(VERSIONS, PACKS, ["@comp-markdown/mermaid@0.1.2"], ["card-v0.2.0", "mermaid-v0.1.2", "v0.9.0"]);
    assert.deepEqual(
      (await toStage(steps)).map((release) => release.pack),
      ["card"],
    );
    assert.deepEqual(log, ["@comp-markdown/card 0.2.0 (card-v0.2.0)"]);
  });

  it("leaves out a pack npm lacks but whose tag isn't at this commit", async () => {
    const { steps } = stubs(VERSIONS, PACKS, [], ["chart-v0.1.0"]);
    assert.deepEqual(
      (await toStage(steps)).map((release) => release.pack),
      ["chart"],
    );
  });

  it("fails with nothing to stage", async () => {
    const { steps } = stubs(VERSIONS, PACKS, ["@comp-markdown/card@0.2.0"], ["card-v0.2.0"]);
    await assert.rejects(toStage(steps), /nothing to stage/);
  });

  it("fails while versions.json and the packs disagree", async () => {
    const { steps } = stubs({ ...VERSIONS, card: "0.3.0" }, PACKS, [], ["card-v0.3.0"]);
    await assert.rejects(toStage(steps), /compmd-packs\/card isn't at 0.3.0/);
  });
});
