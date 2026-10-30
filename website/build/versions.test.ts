// @vitest-environment node
import { describe, expect, it } from "vitest";
import { releases, rootOf, specPages } from "./versions.ts";

describe("specPages", () => {
  it("is the draft alone until a release", () => {
    expect(specPages(["v0.1.0", "spec-draft"])).toEqual([{ folder: "spec/" }]);
  });

  it("shows the latest release at spec/, the draft at spec/draft/, and each release at its own folder", () => {
    expect(specPages(["spec-v1.0", "v0.2.0", "spec-v1.10", "spec-v1.2"])).toEqual([
      { folder: "spec/", version: "1.10", ref: "spec-v1.10" },
      { folder: "spec/draft/" },
      { folder: "spec/1.10/", version: "1.10", ref: "spec-v1.10" },
      { folder: "spec/1.2/", version: "1.2", ref: "spec-v1.2" },
      { folder: "spec/1.0/", version: "1.0", ref: "spec-v1.0" },
    ]);
  });
});

describe("releases", () => {
  it("orders by number, not by text, and drops other tags", () => {
    expect(releases(["spec-v2.0", "spec-v1.9", "spec-v10.0", "spec-v1.x"]).map((r) => r.version)).toEqual(["10.0", "2.0", "1.9"]);
  });
});

describe("rootOf", () => {
  it("goes up a folder per folder", () => {
    expect(rootOf("")).toBe("");
    expect(rootOf("spec/")).toBe("../");
    expect(rootOf("spec/1.0/")).toBe("../../");
  });
});
