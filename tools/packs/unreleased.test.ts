import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { tagOf, unreleased } from "./unreleased.ts";

describe("the packs to release", () => {
  it("are those whose version npm doesn't have, in the packs' order, each with its tag", async () => {
    const asked: string[] = [];
    const has = (name: string, version: string) => {
      asked.push(`${name}@${version}`);
      return Promise.resolve(name === "@comp-markdown/mermaid");
    };
    assert.deepEqual(await unreleased({ wireframe: "0.2.0", mermaid: "0.1.2", card: "0.1.0" }, has), [
      { pack: "card", name: "@comp-markdown/card", version: "0.1.0", tag: "card-v0.1.0" },
      { pack: "wireframe", name: "@comp-markdown/wireframe", version: "0.2.0", tag: "wireframe-v0.2.0" },
    ]);
    assert.deepEqual(asked, ["@comp-markdown/card@0.1.0", "@comp-markdown/mermaid@0.1.2", "@comp-markdown/wireframe@0.2.0"]);
  });

  it("are tagged as the packs' releases always were", () => {
    assert.equal(tagOf("mermaid", "0.2.0"), "mermaid-v0.2.0");
  });
});
