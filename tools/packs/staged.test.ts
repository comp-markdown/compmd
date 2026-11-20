import assert from "node:assert/strict";
import { describe as suite, it } from "node:test";
import { describe, stagedPacks, waiting } from "./staged.ts";

const ITEMS = [
  { id: "c", packageName: "@comp-markdown/chart", version: "0.1.0", status: "pending", createdAt: "2026-11-20T21:05:00Z" },
  { id: "x", packageName: "@compmd/render", version: "0.10.0", createdAt: "2026-11-20T20:00:00Z" },
  { id: "a", packageName: "@comp-markdown/card", version: "0.2.0", createdAt: "2026-11-20T21:04:00Z" },
  { id: "m", packageName: "@comp-markdown/mermaid", version: "0.1.3", status: "approved", createdAt: "2026-11-19T10:00:00Z" },
  { packageName: "@comp-markdown/broken", version: "1.0.0" },
];

suite("what npm has staged of the packs", () => {
  it("is the packs' versions, oldest first, the others' and anything without an id left out", () => {
    assert.deepEqual(
      stagedPacks(JSON.stringify(ITEMS)).map((item) => item.id),
      ["m", "a", "c"],
    );
  });

  it("fails on anything but a list", () => {
    assert.throws(() => stagedPacks('{"error": "E401"}'), /gave no list/);
  });

  it("waits for the author unless its status says it is settled", () => {
    const [mermaid, card, chart] = stagedPacks(JSON.stringify(ITEMS));
    assert.deepEqual([mermaid, card, chart].map((item) => item && waiting(item)), [false, true, true]);
    for (const status of ["rejected", "Published", "APPROVED"]) assert.equal(waiting({ id: "i", packageName: "p", version: "1.0.0", status }), false, status);
  });

  it("is shown with its package, version and when it was staged, and a settled one's status", () => {
    const [mermaid, card] = stagedPacks(JSON.stringify(ITEMS));
    assert.equal(card && describe(card), "@comp-markdown/card 0.2.0, staged 2026-11-20 21:04");
    assert.equal(mermaid && describe(mermaid), "@comp-markdown/mermaid 0.1.3, staged 2026-11-19 10:00 (approved)");
  });
});
