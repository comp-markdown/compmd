import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { npmCommand, npmError } from "./npm.ts";

describe("npm, as the tools run it", () => {
  it("is the pinned version, through npx", () => {
    assert.deepEqual(npmCommand(["stage", "list"]), ["npx", "--yes", "npm@11.21.1", "stage", "list"]);
  });

  it("says what went wrong from its JSON, and how to log in when it isn't", () => {
    const unauthorized = JSON.stringify({ error: { code: "E401", summary: "Unable to authenticate, your authentication token seems to be invalid." } });
    assert.equal(npmError(unauthorized), "Unable to authenticate, your authentication token seems to be invalid. Log in from outside the repository: (cd ~ && npm login)");
    assert.equal(npmError(JSON.stringify({ error: { code: "E404", summary: "Not found" } })), "Not found");
  });

  it("points to its own message when it gave no JSON", () => {
    assert.equal(npmError(undefined), "see npm's message above");
    assert.equal(npmError("not json"), "see npm's message above");
  });
});
