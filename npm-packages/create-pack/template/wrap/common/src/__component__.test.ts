import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./__component__.ts";

describe("__component__", () => {
  it("draws in its box, and goes when destroyed", async () => {
    const shown = mountForTest(mount, { params: __params__, fallback: "__component__, as text" });
    await shown.settled();
    expect(shown.target.childElementCount).toBeGreaterThan(0);
    shown.destroy();
    expect(shown.target.childElementCount).toBe(0);
  });
});
