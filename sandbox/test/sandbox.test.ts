// `createSandbox` (docs/sandbox.md, Q1, decision 2): one document in frame mode and in host mode, and a host's page
// that is itself in a sandboxed frame.

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

/** Opens `tree` in `page` mode, and collects what its view reports. */
function open(page: "frame" | "host") {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "400px";
  areas.push(area);
  const view = createSandbox({ base: frames, page, trusted: { stars } }).open(area, { components });
  views.push(view);
  const events: { [T in keyof ViewEvents]: ViewEvents[T][] } = { set: [], link: [], error: [], rendered: [] };
  const waiters: (() => void)[] = [];
  for (const type of ["set", "link", "error", "rendered"] as const) {
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
  view.render(tree);
  return { area, view, events, until };
}

describe("createSandbox", () => {
  it("renders the page in one strict frame, where only strict components run (frame mode)", async () => {
    const { area, events, until } = open("frame");
    await until(() => events.set.length >= 1 && events.error.length >= 2 && events.rendered.length >= 1);
    expect(events.set).toEqual([{ input: "chart", value: JSON.stringify({ origin: "null" }) }]);
    expect(events.error.map(({ component, reason }) => [component, reason]).sort()).toEqual([
      ["map", "not-here"],
      ["stars", "not-here"],
    ]);
    expect(area.querySelectorAll("iframe")).toHaveLength(1);
  });

  it("renders the page in the host's page, trusted inline and the rest in frames (host mode)", async () => {
    const { area, events, until } = open("host");
    await until(() => events.set.length >= 3 && events.rendered.length >= 1);
    expect(Object.fromEntries(events.set.map(({ input, value }) => [input, JSON.parse(value as string) as unknown]))).toEqual({
      chart: { origin: "null" },
      map: { origin: "null" },
      stars: { origin: "host" },
    });
    expect(events.error).toEqual([]);
    expect(area.querySelectorAll("iframe")).toHaveLength(2);
    expect(area.textContent).toContain("★★★★☆");
  });

  it("works when the host's page is itself in a sandboxed frame (decision 2)", async () => {
    for (const page of ["host", "frame"]) {
      const host = Object.assign(document.createElement("iframe"), { src: `${frames}host/host.html?page=${page}` });
      host.setAttribute("sandbox", "allow-scripts");
      const message = new Promise<unknown>((resolve) => {
        addEventListener("message", function listen({ source, data }: MessageEvent<{ from?: string }>) {
          if (source !== host.contentWindow || data.from !== "sandboxed-host") return;
          removeEventListener("message", listen);
          resolve(data);
        });
      });
      document.body.append(host);
      expect(await message, page).toEqual({ from: "sandboxed-host", input: "result", value: JSON.stringify({ origin: "null", nested: true }) });
      host.remove();
    }
  });
});
