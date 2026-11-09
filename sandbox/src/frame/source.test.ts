import { describe, expect, it } from "vitest";
import { frameSource, withSource } from "./source.ts";

describe("the frames' own source", () => {
  it("is added after each 'self', and nothing changes without one", () => {
    const policy = "script-src 'self' blob:; style-src 'self' 'unsafe-inline'; font-src 'self'";
    expect(withSource(policy, "app://localhost/frames/")).toBe(
      "script-src 'self' app://localhost/frames/ blob:; style-src 'self' app://localhost/frames/ 'unsafe-inline'; font-src 'self' app://localhost/frames/",
    );
    expect(withSource(policy, undefined)).toBe(policy);
  });

  it("takes a scheme, a host and a path, and nothing that would widen a policy", () => {
    for (const good of ["app://localhost/frames/", "figusercontent://localhost/frames/", "http://figusercontent.localhost/frames/", "https://cdn.example.com:8443/compmd/"]) {
      expect(frameSource(good)).toBe(good);
    }
    for (const wrong of ["*", "app:", "app://*", "app://localhost/frames/; script-src *", "'self'", "app://localhost/a b", "app://*.example.com/", "//localhost/frames/"]) {
      expect(() => frameSource(wrong), wrong).toThrow("isn't a source");
    }
  });
});
