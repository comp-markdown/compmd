// The host runtime (docs/sandbox.md, decisions 1, 4, 5, 7, T4, T9, Q2): a document's uses in the host's page, trusted
// inline and every other one in a frame of its level, with its code compiled in the document's compile frame.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, Mount } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { HostRuntime, type HostEvents } from "../src/host/runtime.ts";
import { allowed, frames, leaf, probeCode } from "./harness.ts";

const runtimes: HostRuntime[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const runtime of runtimes.splice(0)) runtime.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** A runtime whose events go to `events`, and a place on the page for its uses. */
function setup(options: Partial<ConstructorParameters<typeof HostRuntime>[0]> = {}) {
  const sets: [string, unknown][] = [];
  const errors: [string, string, string][] = [];
  const waiting = new Map<string, (value: unknown) => void>();
  const events: HostEvents = {
    set: (input, value) => {
      sets.push([input, value]);
      waiting.get(input)?.(JSON.parse(value as string));
    },
    link: () => undefined,
    error: (component, reason, message) => {
      errors.push([component, reason, message]);
      waiting.get(`error:${component}`)?.([reason, message]);
    },
  };
  const runtime = new HostRuntime({ base: frames, timeout: 1500, ...options }, events);
  runtimes.push(runtime);
  const area = document.body.appendChild(document.createElement("div"));
  areas.push(area);
  /** Resolves with the JSON a component sets `input` to, or with an error's reason and message for `error:name`. */
  const next = (key: string) => new Promise((resolve) => waiting.set(key, resolve));
  return { runtime, area, sets, errors, next };
}

/** A leaf use whose `result` param is bound to the input `input`. */
const use = (name: string, input = name): ExpandedComponent => ({ ...leaf(name), params: [{ key: "result", breakpoint: null, value: "", input }] });

const plain = (name: string, trust: HostComponent["trust"], body: string): HostComponent => ({ name, language: "js", code: probeCode(body), trust });

describe("the host runtime", () => {
  it("mounts trusted components inline, and frames the others at their levels, with their code compiled", async () => {
    const stars: Mount = (target, { set }) => {
      target.textContent = "★★★★☆";
      set("result", JSON.stringify({ here: "host" }));
    };
    const { runtime, area, next } = setup({ trusted: { stars } });
    await runtime.set([
      { name: "stars", language: "js", code: "", trust: { level: "trusted" } },
      {
        name: "chart",
        language: "jsx",
        code: `import { useEffect } from "react";
export default function Chart({ compmd }) {
  useEffect(() => { compmd.set("result", JSON.stringify({ origin: self.origin, react: true })); }, []);
  return <svg role="img" aria-label="chart" />;
}`,
        trust: { level: "strict" },
      },
      plain("map", { level: "permissive", network: [allowed.slice(0, -1)], host: false }, `async () => ({ origin: self.origin, reached: await fetch(${JSON.stringify(allowed)} + "attempt?host-test").then(() => true, () => false) })`),
    ]);
    const inputs = { stars: { type: "string", value: "" }, chart: { type: "string", value: "" }, map: { type: "string", value: "" } };
    const results = Promise.all([next("stars"), next("chart"), next("map")]);
    for (const name of ["stars", "chart", "map"]) runtime.mountUse(area.appendChild(document.createElement("div")), use(name), inputs);
    expect(await results).toEqual([{ here: "host" }, { origin: "null", react: true }, { origin: "null", reached: true }]);
    expect(area.querySelectorAll("iframe")).toHaveLength(2);
    expect(area.textContent).toContain("★★★★☆");
  });

  it("gives `host` only to permissive frames of packs given it", async () => {
    const { runtime, area, next } = setup({ host: { rating: (product) => ({ product, stars: 4 }) } });
    const body = `async (target, { host }) => host === undefined ? "none" : await host.call("rating", "p-1")`;
    await runtime.set([
      plain("summary", { level: "permissive", network: [], host: true }, body),
      plain("map", { level: "permissive", network: [], host: false }, body),
      plain("chart", { level: "strict" }, body),
    ]);
    const inputs = { summary: { type: "string", value: "" }, map: { type: "string", value: "" }, chart: { type: "string", value: "" } };
    const results = Promise.all([next("summary"), next("map"), next("chart")]);
    for (const name of ["summary", "map", "chart"]) runtime.mountUse(area.appendChild(document.createElement("div")), use(name), inputs);
    expect(await results).toEqual([{ product: "p-1", stars: 4 }, "none", "none"]);
  });

  it("shows the fallback of a permissive or trusted use inside a strict container, and says why (Q2)", async () => {
    const { runtime, area, next } = setup({ trusted: { stars: () => undefined } });
    await runtime.set([
      { name: "tabs", language: "js", code: "export default function mount(target, { slots }) { for (const item of slots.default) target.append(item.element); }", trust: { level: "strict" } },
      plain("map", { level: "permissive", network: [], host: false }, "async () => 'ran'"),
      { name: "stars", language: "js", code: "", trust: { level: "trusted" } },
    ]);
    const tabs: ExpandedComponent = { type: "componentContainer", name: "tabs", params: [], children: [use("map"), use("stars")] };
    const reasons = Promise.all([next("error:map"), next("error:stars")]);
    runtime.mountUse(area, tabs, {});
    expect(await reasons).toEqual([
      ["not-here", "`map` is permissive and can't be in a strict frame: only the host's page holds permissive frames"],
      ["not-here", "`stars` is trusted and can't be in a strict frame: only the host's page runs trusted code"],
    ]);
  });

  it("says which component didn't compile, and shows its fallback", async () => {
    const { runtime, errors } = setup();
    await runtime.set([{ name: "broken", language: "jsx", code: "export default () => <b>;", trust: { level: "strict" } }]);
    expect(errors).toEqual([["broken", "failed", expect.any(String)]]);
    expect(runtime.code).toEqual([]);
  });

  // WebKit runs these frames in the page's own process, so a frame that spins freezes the host's page with it, and
  // nothing in the page can stop it (docs/sandbox.md, what a browser can't stop).
  const webkit = !navigator.userAgent.includes("Chrome");

  it.skipIf(webkit)("stops a frame that stops answering, while the host's page goes on (T9)", async () => {
    const { runtime, area, next } = setup();
    // It spins for longer than the timeout, then lets go: frames of one site share a process, which the tests after it need.
    await runtime.set([plain("spinner", { level: "strict" }, "async () => { setTimeout(() => { for (const end = Date.now() + 4000; Date.now() < end; ) {} }, 200); return 'started'; }")]);
    const started = next("spinner");
    const stopped = next("error:spinner");
    runtime.mountUse(area, use("spinner"), { spinner: { type: "string", value: "" } });
    expect(await started).toBe("started");
    // The host's page keeps running its own timers while the frame spins.
    let ticks = 0;
    const ticking = setInterval(() => (ticks += 1), 100);
    expect(await stopped).toEqual(["threw", "its frame stopped answering"]);
    clearInterval(ticking);
    expect(ticks).toBeGreaterThan(5);
    expect(area.querySelector("iframe")).toBeNull();
    expect(area.textContent).toBe("spinner's fallback");
  });

  it("keeps one document's frames from reading another's (T4)", async () => {
    const one = setup();
    const other = setup();
    await other.runtime.set([plain("secret", { level: "strict" }, "async (target) => { target.textContent = 'secret data'; return 'shown'; }")]);
    const shown = other.next("secret");
    other.runtime.mountUse(other.area, use("secret"), {});
    expect(await shown).toBe("shown");
    await one.runtime.set([
      plain(
        "reader",
        { level: "strict" },
        `async () => {
          const read = [];
          for (let i = 0; i < parent.frames.length; i++) {
            if (parent.frames[i] === window) continue;
            try { read.push(parent.frames[i].document.body.textContent); } catch { read.push("refused"); }
          }
          return read;
        }`,
      ),
    ]);
    const read = one.next("reader");
    one.runtime.mountUse(one.area, use("reader"), {});
    const got = (await read) as string[];
    expect(got.length).toBeGreaterThan(0);
    expect(got.every((entry) => entry === "refused")).toBe(true);
  });
});
