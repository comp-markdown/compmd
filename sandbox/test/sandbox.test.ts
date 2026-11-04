// `createSandbox` (docs/running-components-in-the-sandbox.md, Q1, decision 2): one document in frame mode and in host
// mode, and a host's page that is itself in a sandboxed frame.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedRoot, Mount } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View, type ViewEvents } from "../src/index.ts";
import { frames, probeCode } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** A document with a strict chart, a permissive map and the host's trusted stars, each setting its input. */
const components: HostComponent[] = [
  { name: "chart", language: "js", code: probeCode("async () => ({ origin: self.origin })"), trust: { level: "strict" } },
  { name: "map", language: "js", code: probeCode("async () => ({ origin: self.origin })"), trust: { level: "permissive", network: [], host: false } },
  { name: "stars", language: "js", code: "", trust: { level: "trusted" } },
];
const stars: Mount = (target, { set }) => {
  target.textContent = "★★★★☆";
  set("result", JSON.stringify({ origin: "host" }));
};
const tree: ExpandedRoot = {
  type: "root",
  children: ["chart", "map", "stars"].map((name) => ({
    type: "componentLeaf" as const,
    name,
    params: [{ key: "result", breakpoint: null, value: "", input: name }],
    fallback: [{ type: "text", value: `${name}'s fallback` }],
  })),
  inputs: { chart: { type: "string", value: "" }, map: { type: "string", value: "" }, stars: { type: "string", value: "" } },
};

/** Opens `root` with `given` in `page` mode, and collects what its view reports. */
function open(page: "frame" | "host", given: HostComponent[] = components, root: ExpandedRoot = tree) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "400px";
  areas.push(area);
  const view = createSandbox({ base: frames, page, trusted: { stars } }).open(area, { components: given });
  views.push(view);
  const events: { [T in keyof ViewEvents]: ViewEvents[T][] } = { set: [], link: [], error: [], recovered: [], rendered: [], loaded: [], pointed: [], layout: [] };
  const waiters: (() => void)[] = [];
  for (const type of ["set", "link", "error", "recovered", "rendered", "loaded"] as const) {
    view.on(type, (event) => {
      (events[type] as unknown[]).push(event);
      for (const waiter of waiters) waiter();
    });
  }
  /** Resolves once `done` holds of the events so far. */
  const until = (done: () => boolean) =>
    new Promise<void>((resolve) => {
      const check = () => {
        if (done()) resolve();
      };
      waiters.push(check);
      check();
    });
  view.render(root);
  return { area, view, events, until };
}

/** Components whose code can't load (an import nothing provides) or throws, and a tree that uses both. */
const broken: HostComponent[] = [
  { name: "unloadable", language: "js", code: 'import * as three from "three";\nexport default (target) => { target.textContent = String(three); };', trust: { level: "strict" } },
  { name: "thrower", language: "js", code: 'export default () => { throw new Error("broken"); };', trust: { level: "strict" } },
];
const brokenTree: ExpandedRoot = {
  type: "root",
  children: broken.map(({ name }) => ({ type: "componentLeaf" as const, name, params: [], fallback: [{ type: "text", value: `${name}'s fallback` }] })),
};

describe("createSandbox", () => {
  it("renders the page in one strict frame, where only strict components run (frame mode)", async () => {
    const { area, events, until } = open("frame");
    await until(() => events.set.length >= 1 && events.error.length >= 2 && events.rendered.length >= 1 && events.loaded.length >= 1);
    expect(events.set).toEqual([{ input: "chart", value: JSON.stringify({ origin: "null" }) }]);
    expect(events.error.map(({ component, reason }) => [component, reason]).sort()).toEqual([
      ["map", "not-here"],
      ["stars", "not-here"],
    ]);
    expect(area.querySelectorAll("iframe")).toHaveLength(1);
  });

  it("renders the page in the host's page, trusted inline and the rest in frames (host mode)", async () => {
    const { area, events, until } = open("host");
    await until(() => events.set.length >= 3 && events.rendered.length >= 1 && events.loaded.length >= 1);
    expect(Object.fromEntries(events.set.map(({ input, value }) => [input, JSON.parse(value as string) as unknown]))).toEqual({
      chart: { origin: "null" },
      map: { origin: "null" },
      stars: { origin: "host" },
    });
    expect(events.error).toEqual([]);
    expect(area.querySelectorAll("iframe")).toHaveLength(2);
    expect(area.textContent).toContain("★★★★☆");
  });

  it("reports code that can't load as W121 and code that throws as W118 (spec §7.6, §10.2)", async () => {
    for (const page of ["frame", "host"] as const) {
      const { events, until } = open(page, broken, brokenTree);
      // Each frame reports its own: a component that fails in two frames is reported by each.
      const reported = () => new Set(events.error.map(({ component, reason, code }) => `${component} ${reason} ${code ?? ""}`));
      await until(() => reported().size >= 2);
      expect([...reported()].sort(), page).toEqual(["thrower threw W118", "unloadable failed W121"]);
    }
  });

  it("works when the host's page is itself in a sandboxed frame, with compiled source and a module it holds (decision 2)", async () => {
    for (const page of ["host", "frame"]) {
      const host = Object.assign(document.createElement("iframe"), { src: `${frames}host/host.html?page=${page}` });
      host.setAttribute("sandbox", "allow-scripts");
      const sets = new Promise<Record<string, unknown>>((resolve) => {
        const got: Record<string, unknown> = {};
        addEventListener("message", function listen({ source, data }: MessageEvent<{ from?: string; input?: string; value?: unknown }>) {
          if (source !== host.contentWindow || data.from !== "sandboxed-host") return;
          got[data.input ?? "error"] = data.input ? data.value : data;
          if (Object.keys(got).length < 2) return;
          removeEventListener("message", listen);
          resolve(got);
        });
      });
      document.body.append(host);
      const ran = JSON.stringify({ origin: "null", nested: true });
      expect(await sets, page).toEqual({ result: ran, diagram: ran });
      host.remove();
    }
  });
});
