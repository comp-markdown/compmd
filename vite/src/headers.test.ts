import { describe, expect, it } from "vitest";
import { headersFile } from "./headers.ts";

describe("headersFile", () => {
  it("gives the frames and the packs under the prefix the contract's headers, and nothing else", () => {
    const headers = [
      "  Content-Security-Policy: sandbox allow-scripts",
      "  Access-Control-Allow-Origin: *",
      "  Access-Control-Expose-Headers: *",
      "  X-Content-Type-Options: nosniff",
      "  Cache-Control: max-age=31536000, immutable",
    ].join("\n");
    expect(headersFile("/compmd/")).toBe(`/compmd/frames/*\n${headers}\n\n/compmd/packs/*\n${headers}\n`);
  });
});
