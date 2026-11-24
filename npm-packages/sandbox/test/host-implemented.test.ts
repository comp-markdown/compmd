// A component the host implements (spec §7.1; docs/running-components-in-the-sandbox.md, decision 11): a signature
// fence alone in an ambient pack, which the processor lists with no language and no code. Only at the trusted level
// does the host's `trusted` mount of its name run it, in the host's page.

import type { HostComponent } from "@compmd/processor";
import type { Mount } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { HostRuntime } from "../src/host/runtime.ts";
import { frames, leaf } from "./harness.ts";

const runtimes: HostRuntime[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const runtime of runtimes.splice(0)) runtime.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** A runtime with the host's `trusted` mounts, the errors it reports, and a place on the page for one use. */
function setup(trusted: Record<string, Mount>) {
  const errors: [string, string, string][] = [];
  const runtime = new HostRuntime({ base: frames, timeout: 1500, compiled: null, trusted }, {
    set: () => undefined,
    link: () => undefined,
    error: (component, reason, message) => {
      errors.push([component, reason, message]);
    },
  });
  runtimes.push(runtime);
  const area = document.body.appendChild(document.createElement("div"));
  areas.push(area);
  return { runtime, area, errors };
}

/** `item-header` as the processor lists it: an ambient pack's, of `trust`, with no language and no code. */
const itemHeader = (trust: HostComponent["trust"]): HostComponent => ({ name: "item-header", code: "", trust, pack: "@acme/app" });

describe("a component the host implements", () => {
  it("mounts the host's trusted mount of its name, in the host's page, when its pack is trusted", async () => {
    const { runtime, area, errors } = setup({
      "item-header": (target) => {
        target.textContent = "The host's header";
      },
    });
    await runtime.set([itemHeader({ level: "trusted" })]);
    runtime.mountUse(area, leaf("item-header"), {});
    expect(area.textContent).toBe("The host's header");
    expect(area.querySelector("iframe")).toBeNull();
    expect(errors).toEqual([]);
  });

  it("runs no host code when its pack is strict: its uses show their fallback, as not-here", async () => {
    let ran = false;
    const { runtime, area, errors } = setup({
      "item-header": () => {
        ran = true;
      },
    });
    await runtime.set([itemHeader({ level: "strict" })]);
    runtime.mountUse(area, leaf("item-header"), {});
    expect(ran).toBe(false);
    expect(area.textContent).toBe("item-header's fallback");
    expect(area.querySelector("iframe")).toBeNull();
    expect(errors).toEqual([["item-header", "not-here", "`item-header` is implemented by the host and its pack is strict: the host's own code runs only at the trusted level, in its page"]]);
    expect(runtime.code).toEqual([{ name: "item-header", level: "strict", pack: "@acme/app" }]);
  });

  it("shows its fallback, reported as missing, when the host has no mount for it", async () => {
    const { runtime, area, errors } = setup({});
    await runtime.set([itemHeader({ level: "trusted" })]);
    runtime.mountUse(area, leaf("item-header"), {});
    expect(area.textContent).toBe("item-header's fallback");
    expect(errors.map(([component, reason]) => [component, reason])).toEqual([["item-header", "missing"]]);
  });
});
