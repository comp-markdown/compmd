import { afterEach, describe, expect, it } from "vitest";
import { givenBase } from "./given-base.ts";
import { createSandbox } from "./sandbox.ts";

const page = globalThis as { __COMPMD__?: unknown };

afterEach(() => {
  delete page.__COMPMD__;
});

describe("the base the host's app gives", () => {
  it("is globalThis.__COMPMD__.base when it is text, and nothing otherwise", () => {
    expect(givenBase()).toBeUndefined();
    page.__COMPMD__ = { base: "app://localhost/frames/1df336e75685de8c/" };
    expect(givenBase()).toBe("app://localhost/frames/1df336e75685de8c/");
    for (const wrong of [{}, { base: 1 }, null, "app://localhost/frames/"]) {
      page.__COMPMD__ = wrong;
      expect(givenBase(), JSON.stringify(wrong)).toBeUndefined();
    }
  });

  it("is what createSandbox needs when no base is given, and without either it says so", () => {
    expect(() => createSandbox({ page: "host" })).toThrow("createSandbox needs the frames' `base`");
    page.__COMPMD__ = { base: "app://localhost/frames/1df336e75685de8c/" };
    expect(() => createSandbox({ page: "host" })).not.toThrow();
  });
});
