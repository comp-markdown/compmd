import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PACKS } from "./approve-packs.ts";
import { RELEASE } from "./approve-release.ts";
import { approveArgs, approveStaged, type Steps } from "./approve-staged.ts";
import type { Staged } from "./staged.ts";

/** `approveStaged` for the packs, as `cargo make approve-packs` runs it. */
const approvePacks = (options: { dryRun: boolean }, steps: Steps) => approveStaged({ ...PACKS, ...options }, steps);

const ITEMS = [
  { id: "c", packageName: "@comp-markdown/chart", version: "0.1.0", createdAt: "2026-11-20T21:05:00Z" },
  { id: "a", packageName: "@comp-markdown/card", version: "0.2.0", createdAt: "2026-11-20T21:04:00Z" },
  { id: "m", packageName: "@comp-markdown/mermaid", version: "0.1.3", status: "approved", createdAt: "2026-11-19T10:00:00Z" },
];

/** Steps over `items`, the author choosing with `pick` (by default what starts chosen) and giving `code`. */
function stubs(items: unknown[], options: { pick?: (offered: readonly Staged[], chosen: (item: Staged) => boolean) => Staged[]; code?: string } = {}) {
  const done: string[] = [];
  const log: string[] = [];
  const offered: string[] = [];
  const steps: Steps = {
    list: () => JSON.stringify(items),
    choose: (list, chosen) => {
      offered.push(...list.map((item) => `${chosen(item) ? "[x]" : "[ ]"} ${item.id}`));
      return Promise.resolve(options.pick ? options.pick(list, chosen) : list.filter(chosen));
    },
    code: () => {
      done.push("code");
      return Promise.resolve(options.code);
    },
    npm: (args) => {
      done.push(args.join(" "));
    },
    log: (line) => {
      log.push(line);
    },
  };
  return { steps, done, log, offered };
}

describe("cargo make approve-packs", () => {
  it("offers every staged pack oldest first, those waiting chosen, and approves the chosen ones with one code", async () => {
    const { steps, done, offered } = stubs(ITEMS, { code: "123456" });
    assert.equal(await approvePacks({ dryRun: false }, steps), 0);
    assert.deepEqual(offered, ["[ ] m", "[x] a", "[x] c"]);
    assert.deepEqual(done, ["code", "stage approve a --otp 123456", "stage approve c --otp 123456"]);
  });

  it("approves in the order they were staged, whatever order the choice comes back in", async () => {
    const { steps, done } = stubs(ITEMS, { pick: (list) => [...list].reverse() });
    await approvePacks({ dryRun: false }, steps);
    assert.deepEqual(done, ["code", "stage approve m", "stage approve a", "stage approve c"]);
  });

  it("approves only what the author left chosen", async () => {
    const { steps, done } = stubs(ITEMS, { pick: (list) => list.filter((item) => item.id === "c") });
    await approvePacks({ dryRun: false }, steps);
    assert.deepEqual(done, ["code", "stage approve c"]);
  });

  it("asks for no code and approves nothing when nothing is chosen", async () => {
    const { steps, done, log } = stubs(ITEMS, { pick: () => [] });
    assert.equal(await approvePacks({ dryRun: false }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(log, ["nothing chosen: nothing approved"]);
  });

  it("leaves the code to npm when the author gives none", () => {
    assert.deepEqual(approveArgs({ id: "a", packageName: "p", version: "1.0.0" }), ["stage", "approve", "a"]);
  });

  it("with --dry-run, lists them as it would offer them, and approves nothing", async () => {
    const { steps, done, log, offered } = stubs(ITEMS);
    assert.equal(await approvePacks({ dryRun: true }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(offered, []);
    assert.deepEqual(log, [
      "staged, 3; would offer them all, these chosen:",
      "  [ ] @comp-markdown/mermaid 0.1.3, staged 2026-11-19 10:00 (approved)",
      "  [x] @comp-markdown/card 0.2.0, staged 2026-11-20 21:04",
      "  [x] @comp-markdown/chart 0.1.0, staged 2026-11-20 21:05",
    ]);
  });

  it("says so when nothing is staged", async () => {
    const { steps, done, log } = stubs([{ id: "x", packageName: "@compmd/render", version: "0.10.0" }]);
    assert.equal(await approvePacks({ dryRun: false }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(log, ["npm has nothing staged for the packs"]);
  });

  it("stops at npm's failure, the versions before it approved", async () => {
    const { steps, done } = stubs(ITEMS);
    steps.npm = (args) => {
      if (args.includes("c")) throw new Error("npm failed");
      done.push(args.join(" "));
    };
    await assert.rejects(approvePacks({ dryRun: false }, steps), /npm failed/);
    assert.deepEqual(done, ["code", "stage approve a"]);
  });
});

describe("cargo make approve-release", () => {
  const RELEASED = [
    { id: "s", packageName: "@compmd/sandbox", version: "0.10.0", createdAt: "2026-11-23T23:40:03Z" },
    { id: "r", packageName: "@compmd/render", version: "0.10.0", createdAt: "2026-11-23T23:40:01Z" },
    { id: "p", packageName: "@compmd/processor", version: "0.10.0", createdAt: "2026-11-23T23:40:02Z" },
    ...ITEMS,
  ];

  it("approves the release's @compmd packages in the order they were staged, and no pack", async () => {
    const { steps, done, offered } = stubs(RELEASED, { code: "123456" });
    assert.equal(await approveStaged({ ...RELEASE, dryRun: false }, steps), 0);
    assert.deepEqual(offered, ["[x] r", "[x] p", "[x] s"]);
    assert.deepEqual(done, ["code", "stage approve r --otp 123456", "stage approve p --otp 123456", "stage approve s --otp 123456"]);
  });

  it("says so when the release has nothing staged", async () => {
    const { steps, done, log } = stubs(ITEMS);
    assert.equal(await approveStaged({ ...RELEASE, dryRun: false }, steps), 0);
    assert.deepEqual(done, []);
    assert.deepEqual(log, ["npm has nothing staged for the release"]);
  });
});
