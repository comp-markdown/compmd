import { describe, expect, it } from "vitest";
import { narrowed, networkPolicy } from "./narrow.ts";

describe("a permissive frame's network", () => {
  it("allows its pack's origins for requests, images, fonts and media", () => {
    expect(networkPolicy(["https://tiles.example.com", "wss://live.example.com"]).split("; ")).toEqual([
      "connect-src https://tiles.example.com wss://live.example.com",
      "img-src data: blob: https://tiles.example.com wss://live.example.com",
      "font-src 'self' https://tiles.example.com wss://live.example.com",
      "media-src https://tiles.example.com wss://live.example.com",
    ]);
  });

  it("allows nothing more than a strict frame without origins", () => {
    expect(networkPolicy([]).split("; ")).toEqual(["connect-src 'none'", "img-src data: blob:", "font-src 'self'", "media-src 'none'"]);
  });

  it("refuses what isn't an origin, so a host's configuration can't widen a policy", () => {
    for (const wrong of ["*", "'unsafe-eval'", "https://a.example; script-src *", "tiles.example.com"]) {
      expect(() => networkPolicy([wrong]), wrong).toThrow("isn't an origin");
    }
  });

  it("narrows the first policy: every other request falls to `default-src 'none'`, and only strict frames nest", () => {
    expect(narrowed("script-src 'self'", "connect-src https://a.example", "https://app.example/compmd/strict.html")).toBe(
      "default-src 'none'; script-src 'self'; connect-src https://a.example; frame-src https://app.example/compmd/strict.html; child-src https://app.example/compmd/strict.html",
    );
  });
});
