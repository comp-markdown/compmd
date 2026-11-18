import { describe, expect, it } from "vitest";
import { readConfig } from "./config.ts";
import { cspFor } from "./csp.ts";

describe("cspFor", () => {
  it("gives the frames from the origin, and Shiki and Temml from its frames, never its packs", () => {
    expect(cspFor("compmd://localhost")).toEqual({
      "frame-src": ["compmd://localhost/"],
      "script-src": ["compmd://localhost/frames/"],
      "connect-src": ["compmd://localhost/frames/"],
      "frame-ancestors": ["'none'"],
    });
  });

  it("takes an app's folder as the origin", () => {
    expect(cspFor("https://app.example/compmd/")["script-src"]).toEqual(["https://app.example/compmd/frames/"]);
  });

  it("adds the registry proxy only when the host fetches from registries", () => {
    const registries = { registries: { "*": "https://registry.npmjs.org/" } };
    expect(cspFor("compmd://localhost", readConfig(registries))["connect-src"]).toEqual(["compmd://localhost/frames/", "compmd://localhost/registry/"]);
    expect(cspFor("compmd://localhost", readConfig({ ...registries, fetch: "direct" }))["connect-src"]).toEqual(["compmd://localhost/frames/"]);
    expect(cspFor("compmd://localhost", readConfig({}))["connect-src"]).toEqual(["compmd://localhost/frames/"]);
  });
});
