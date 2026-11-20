import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { packAt } from "./testing.ts";
import { differing, documentVersion, errors, manifestVersion, withVersion } from "./versions.ts";

describe("a pack's versions", () => {
  it("are read from package.json and from the document's export.version", () => {
    const pack = packAt("card", "0.1.0", "0.0.9");
    assert.equal(manifestVersion(pack.manifest), "0.1.0");
    assert.equal(documentVersion(pack.document), "0.0.9");
  });

  it("are written into both, the rest of each file as it was", () => {
    const pack = packAt("card", "0.1.0");
    const next = withVersion(pack, "0.2.0");
    assert.equal(next.manifest, pack.manifest.replace('"version": "0.1.0"', '"version": "0.2.0"'));
    assert.equal(next.document, pack.document.replace('version: "0.1.0"', 'version: "0.2.0"'));
  });

  it("aren't read from a version key outside export", () => {
    const document = `---\nmd:\n  version: 1\n  export:\n    pack: "@comp-markdown/card"\n    version: "0.3.0"\n---\n`;
    assert.equal(documentVersion(document), "0.3.0");
  });
});

describe("versions.json against the packs", () => {
  it("finds a pack it doesn't name, a name that isn't a pack, a bad version and a missing export.version", () => {
    const noExport = { ...packAt("chart", "0.1.0"), document: "# chart\n" };
    assert.deepEqual(errors({ card: "0.1.0", gone: "1.0.0", chart: "next" }, [packAt("card", "0.1.0"), noExport, packAt("music", "0.1.0")]), [
      "compmd-packs/music has no version in compmd-packs/versions.json",
      "compmd-packs/versions.json names gone, which isn't a pack in compmd-packs/",
      'compmd-packs/versions.json gives chart "next", which isn\'t a version',
      "compmd-packs/chart/index.comp.md has no export.version",
    ]);
  });

  it("finds the packs whose files don't have their version, with what they have", () => {
    const behind = differing({ card: "0.2.0", chart: "0.1.0", music: "0.1.0" }, [packAt("card", "0.1.0"), packAt("chart", "0.1.0"), packAt("music", "0.1.0", "0.0.1")]);
    assert.deepEqual(
      behind.map(({ pack, version, written }) => [pack.name, version, written]),
      [
        ["card", "0.2.0", ["0.1.0", "0.1.0"]],
        ["music", "0.1.0", ["0.0.1"]],
      ],
    );
  });

  it("finds nothing when they agree", () => {
    const packs = [packAt("card", "0.1.0"), packAt("chart", "1.0.0-beta.1")];
    assert.deepEqual(errors({ card: "0.1.0", chart: "1.0.0-beta.1" }, packs), []);
    assert.deepEqual(differing({ card: "0.1.0", chart: "1.0.0-beta.1" }, packs), []);
  });
});
