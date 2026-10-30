// Built modules (docs/sandbox.md, T8, decision 7): a frame loads a pack's module by its URL, and runs it only if its
// file matches the hash the host gives.

import { afterEach, describe, expect, it } from "vitest";
import { frames, leaf, probe, type Probe } from "./harness.ts";

let probes: Probe[] = [];
afterEach(() => {
  for (const { frame } of probes) frame.destroy();
  probes = [];
});

/** The file's SHA-384, as an integrity hash. */
async function integrityOf(url: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-384", await (await fetch(url)).arrayBuffer());
  return `sha384-${btoa(String.fromCharCode(...new Uint8Array(digest)))}`;
}

describe("a built module", () => {
  const url = () => `${frames}packs/built.js`;

  it("loads by its URL and runs when its file matches its hash", async () => {
    const started = probe("strict", "", { code: [{ name: "probe", level: "strict", module: { url: url(), integrity: await integrityOf(url()) } }] });
    probes.push(started);
    expect(await started.result).toBe("built module");
  });

  it("doesn't run when its file doesn't match, and its use shows its fallback", async () => {
    const errors: unknown[] = [];
    let failed: () => void = () => undefined;
    const reported = new Promise<void>((resolve) => (failed = resolve));
    const started = probe("strict", "", {
      code: [{ name: "probe", level: "strict", module: { url: url(), integrity: "sha384-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" } }],
      node: leaf("probe"),
      on: {
        error: (error) => {
          errors.push(error);
          failed();
        },
        set: () => {
          errors.push("ran");
        },
      },
    });
    probes.push(started);
    await reported;
    expect(errors).toEqual([expect.objectContaining({ reason: "failed", component: "probe" })]);
  });
});
