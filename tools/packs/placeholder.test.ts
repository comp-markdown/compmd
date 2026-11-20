import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { isPackName, packageOf, placeholderFiles } from "./placeholder.ts";

describe("a pack's placeholder", () => {
  const files = placeholderFiles("card", "MIT License\n");
  const manifest = JSON.parse(files["package.json"] ?? "") as Record<string, unknown>;

  it("is 0.0.0 of the pack's package, with no code", () => {
    assert.deepEqual(Object.keys(files).sort(), ["LICENSE", "README.md", "package.json"]);
    assert.equal(manifest.name, "@comp-markdown/card");
    assert.equal(manifest.version, "0.0.0");
    assert.deepEqual(manifest.files, ["README.md", "LICENSE"]);
    assert.equal(manifest.main, undefined);
  });

  it("names the repository and the pack's folder, which the trusted publisher's provenance checks", () => {
    assert.deepEqual(manifest.repository, { type: "git", url: "git+https://github.com/comp-markdown/compmd.git", directory: "compmd-packs/card" });
  });

  it("is MIT, with the licence it is given, and says it is a placeholder", () => {
    assert.equal(manifest.license, "MIT");
    assert.equal(files.LICENSE, "MIT License\n");
    assert.match(String(manifest.description), /placeholder with no code/);
    assert.match(files["README.md"] ?? "", /^# @comp-markdown\/card\n/);
    assert.match(files["README.md"] ?? "", /compmd-packs\/card\)/);
  });
});

describe("a pack's name", () => {
  it("is lowercase words joined by -", () => {
    for (const name of ["card", "embed-youtube", "qr2"]) assert.equal(isPackName(name), true, name);
    for (const name of ["", "Card", "embed_youtube", "-card", "card-", "a--b", "@comp-markdown/card", "../card"]) assert.equal(isPackName(name), false, name);
  });

  it("gives its package in the packs' scope", () => {
    assert.equal(packageOf("embed-youtube"), "@comp-markdown/embed-youtube");
  });
});
