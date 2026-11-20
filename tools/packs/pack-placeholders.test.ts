import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { placeholders, publishArgs, trustArgs, type Steps } from "./pack-placeholders.ts";

/** Steps that record what they were asked, with npm having `published`. */
function stubs(published: readonly string[]) {
  const done: string[] = [];
  const log: string[] = [];
  const written: Record<string, string>[] = [];
  const steps: Steps = {
    has: (name) => Promise.resolve(published.includes(name)),
    write(files) {
      written.push(files);
      return `/tmp/placeholder-${String(written.length)}`;
    },
    npm: (args) => {
      done.push(args.join(" "));
    },
    pause: () => {
      done.push("pause");
      return Promise.resolve();
    },
    log: (line) => {
      log.push(line);
    },
  };
  return { steps, done, log, written };
}

describe("cargo make pack-placeholders", () => {
  it("publishes a placeholder for each pack npm doesn't have, then sets its trusted publisher, pausing between", async () => {
    const { steps, done, written } = stubs(["@comp-markdown/mermaid"]);
    assert.equal(await placeholders(["card", "mermaid", "chart"], "MIT", { dryRun: false }, steps), 0);
    assert.deepEqual(done, [
      publishArgs("/tmp/placeholder-1").join(" "),
      "pause",
      trustArgs("@comp-markdown/card").join(" "),
      "pause",
      publishArgs("/tmp/placeholder-2").join(" "),
      "pause",
      trustArgs("@comp-markdown/chart").join(" "),
    ]);
    assert.deepEqual(
      written.map((files) => (JSON.parse(files["package.json"] ?? "") as { name: string }).name),
      ["@comp-markdown/card", "@comp-markdown/chart"],
    );
  });

  it("sets the publisher to this repository's release-packs.yml, staging only", () => {
    assert.deepEqual(trustArgs("@comp-markdown/card"), ["trust", "github", "@comp-markdown/card", "--file", "release-packs.yml", "--repository", "comp-markdown/compmd", "--allow-stage-publish", "--yes"]);
    assert.deepEqual(publishArgs("/tmp/x"), ["publish", "/tmp/x", "--access", "public"]);
  });

  it("with --dry-run, says what it would run for a pack npm doesn't have, and does nothing", async () => {
    const { steps, done, log, written } = stubs(["@comp-markdown/mermaid"]);
    assert.equal(await placeholders(["card", "mermaid"], "MIT", { dryRun: true }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(written, []);
    assert.deepEqual(log, [
      "@comp-markdown/card: would publish 0.0.0 (package.json, README.md, LICENSE), then set its trusted publisher:",
      "  npx --yes npm@11.21.1 publish <a new folder> --access public",
      "  npx --yes npm@11.21.1 trust github @comp-markdown/card --file release-packs.yml --repository comp-markdown/compmd --allow-stage-publish --yes",
      "@comp-markdown/mermaid: npm has it already, left as it is",
    ]);
  });

  it("says to release within 2 days once it created any, and nothing when npm had them all", async () => {
    const created = stubs([]);
    await placeholders(["card"], "MIT", { dryRun: false }, created.steps);
    assert.match(created.log.at(-1) ?? "", /within 2 days/);
    const none = stubs(["@comp-markdown/card"]);
    await placeholders(["card"], "MIT", { dryRun: false }, none.steps);
    assert.doesNotMatch(none.log.join("\n"), /2 days/);
  });

  it("does nothing for a name that isn't a pack's, or none", async () => {
    for (const packs of [["card", "Card"], ["../x"], []]) {
      const { steps, done } = stubs([]);
      assert.equal(await placeholders(packs, "MIT", { dryRun: false }, steps), 2);
      assert.deepEqual(done, []);
    }
  });

  it("stops at npm's failure, the packs before it done", async () => {
    const { steps, done } = stubs([]);
    steps.npm = (args) => {
      if (args.includes("@comp-markdown/chart")) throw new Error("npm failed");
      done.push(args[0] ?? "");
    };
    await assert.rejects(placeholders(["card", "chart", "music"], "MIT", { dryRun: false }, steps), /npm failed/);
    assert.deepEqual(done, ["publish", "pause", "trust", "pause", "publish", "pause"]);
  });
});
