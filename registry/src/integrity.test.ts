import { describe, expect, it } from "vitest";
import { integrityOf as nodeIntegrity, tarballOf } from "../test/local-registry.ts";
import { integrityOf, isChecked } from "./integrity.ts";

describe("integrity", () => {
  it("is npm's: sha512 of the tarball, in base64", async () => {
    const bytes = tarballOf("kit-report-1.0.0.tgz");
    expect(await integrityOf(new Uint8Array(bytes))).toBe(nodeIntegrity(bytes));
  });

  it("is checked only when it is a SHA-512", () => {
    expect(isChecked("sha512-x")).toBe(true);
    expect(isChecked("sha1-x")).toBe(false);
    expect(isChecked(undefined)).toBe(false);
  });
});
