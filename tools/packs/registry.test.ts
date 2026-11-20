import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { packageUrl, versionsOf, type Fetch } from "./registry.ts";

/** A registry that answers `status` with `body` for every package, and records what it was asked. */
function registry(status: number, body: unknown = {}) {
  const asked: string[] = [];
  const fetch: Fetch = (url) => {
    asked.push(url);
    return Promise.resolve({ status, ok: status >= 200 && status < 300, json: () => Promise.resolve(body) });
  };
  return { fetch, asked };
}

describe("what the registry has of a package", () => {
  it("is its versions, asked at its address with the scope's slash escaped", async () => {
    const { fetch, asked } = registry(200, { versions: { "0.0.0": {}, "0.1.0": {} } });
    assert.deepEqual(await versionsOf("@comp-markdown/mermaid", fetch), ["0.0.0", "0.1.0"]);
    assert.deepEqual(asked, ["https://registry.npmjs.org/@comp-markdown%2fmermaid"]);
    assert.equal(packageUrl("@comp-markdown/card"), "https://registry.npmjs.org/@comp-markdown%2fcard");
  });

  it("is nothing for a package it doesn't have", async () => {
    assert.deepEqual(await versionsOf("@comp-markdown/card", registry(404).fetch), []);
  });

  it("fails on any other answer, rather than guessing", async () => {
    await assert.rejects(versionsOf("@comp-markdown/card", registry(503).fetch), /answered 503 for @comp-markdown\/card/);
  });
});
