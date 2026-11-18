// @vitest-environment happy-dom
// Where a sandbox's frames and modules are: the bundle's paths under the origin the host's adapter gives, or a base.
import { afterEach, describe, expect, it } from "vitest";
import { found, NO_FRAMES } from "./found.ts";
import { givenOrigin } from "./given-origin.ts";
import { createSandbox } from "./sandbox.ts";

const page = globalThis as { __COMPMD__?: unknown };

afterEach(() => {
  delete page.__COMPMD__;
});

const FRAMES = "frames/1df336e75685de8c/";
const place = { pack: "@kit/bars", version: "1.0.0", path: "dist/bars.js" };

describe("the origin the host's adapter gives", () => {
  it("is globalThis.__COMPMD__.origin when it is text, and nothing otherwise", () => {
    expect(givenOrigin()).toBeUndefined();
    page.__COMPMD__ = { origin: "app://localhost" };
    expect(givenOrigin()).toBe("app://localhost");
    for (const wrong of [{}, { origin: 1 }, { base: "app://localhost/frames/" }, null, "app://localhost"]) {
      page.__COMPMD__ = wrong;
      expect(givenOrigin(), JSON.stringify(wrong)).toBeUndefined();
    }
  });

  it("puts the bundle's frames under it, and createSandbox needs that or a base, saying so", () => {
    expect(() => createSandbox({ page: "host", frames: FRAMES })).toThrow(NO_FRAMES);
    page.__COMPMD__ = { origin: "app://localhost" };
    expect(found({ frames: FRAMES }).base).toBe(`app://localhost/${FRAMES}`);
    expect(() => createSandbox({ page: "host", frames: FRAMES })).not.toThrow();
    expect(() => createSandbox({ page: "host" }), "no frames named").toThrow(NO_FRAMES);
  });
});

describe("where the frames and modules are", () => {
  it("takes an origin given in the options over the adapter's, and one relative to the page", () => {
    page.__COMPMD__ = { origin: "app://localhost" };
    expect(found({ origin: "https://frames.example", frames: FRAMES }).base).toBe(`https://frames.example/${FRAMES}`);
    expect(found({ origin: "/compmd", frames: FRAMES }).base).toBe(new URL(`/compmd/${FRAMES}`, location.href).href);
  });

  it("takes a base as it is, for frames a host built itself", () => {
    expect(found({ base: "/sandbox/" }).base).toBe("/sandbox/");
  });

  it("makes a module's path under the origin a URL, and leaves a URL or a file as it is", () => {
    const code = new Blob(["export default () => {};"]);
    const answers = [{ path: "packs/@kit/bars@1.0.0/dist/bars.js", integrity: "sha384-a" }, { url: "https://cdn.example/bars.js", integrity: "sha384-b" }, { code }, undefined];
    const { modules } = found({ origin: "app://localhost", frames: FRAMES, modules: () => answers.shift() });
    expect(modules?.(place)).toEqual({ url: "app://localhost/packs/@kit/bars@1.0.0/dist/bars.js", integrity: "sha384-a" });
    expect(modules?.(place)).toEqual({ url: "https://cdn.example/bars.js", integrity: "sha384-b" });
    expect(modules?.(place)).toEqual({ code });
    expect(modules?.(place)).toBeUndefined();
    expect(found({ base: "/sandbox/" }).modules, "no modules given").toBeUndefined();
  });

  it("has no URL for a module's path without an origin", () => {
    const { modules } = found({ base: "/sandbox/", modules: () => ({ path: "packs/x", integrity: "sha384-a" }) });
    expect(modules?.(place)).toBeUndefined();
  });
});
