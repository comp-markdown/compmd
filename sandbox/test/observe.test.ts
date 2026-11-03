// The sandbox's observer in the browser (docs/devtools.md, decisions 5 and 6, O2 to O7): the whole tree in both modes,
// what frames report marked as their report, a forged report acted on by nothing, an unasked one dropped, a flood
// bounded, and the same messages with the observer as without it.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, ExpandedRoot, Mount } from "@compmd/render";
import { afterEach, describe, expect, it, vi } from "vitest";
import { createSandbox, type Observation, type ObservedEvent, type View } from "../src/index.ts";
import { frames, pause, probeCode } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
const restores: (() => void)[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
  for (const restore of restores.splice(0)) restore();
});

const use = (name: string, children?: ExpandedComponent[]): ExpandedComponent =>
  children
    ? { type: "componentContainer", name, params: [], children }
    : { type: "componentLeaf", name, params: [{ key: "result", breakpoint: null, value: "", input: name }], fallback: [{ type: "text", value: `${name}'s fallback` }] };

const treeOf = (children: ExpandedComponent[], inputs: string[]): ExpandedRoot => ({
  type: "root",
  children,
  inputs: Object.fromEntries(inputs.map((name) => [name, { type: "string" as const, value: "" }])),
});

const probing = (name: string, trust: HostComponent["trust"], body = "async () => self.origin"): HostComponent => ({ name, language: "js", code: probeCode(body), trust });

/** A permissive map that shows its slot, with a strict legend in it, beside a strict chart and the host's stars. */
const nested: HostComponent[] = [
  probing("chart", { level: "strict" }),
  { name: "map", language: "js", code: "export default (target, { slots }) => { for (const item of slots.default) target.append(item.element); }", trust: { level: "permissive", network: [], host: false } },
  probing("legend", { level: "strict" }),
  { name: "stars", language: "js", code: "", trust: { level: "trusted" } },
];
const nestedTree = treeOf([use("chart"), use("map", [use("legend")]), use("stars")], ["chart", "legend", "stars"]);
const stars: Mount = (target) => {
  target.textContent = "★★★★☆";
};

/** Opens `tree` in `page` mode, observed from the start unless `observe` is false. */
function open(page: "frame" | "host", components: HostComponent[], tree: ExpandedRoot, observe = true) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "400px";
  areas.push(area);
  const view = createSandbox({ base: frames, page, trusted: { stars } }).open(area, { components });
  views.push(view);
  const sets = new Map<string, unknown>();
  view.on("set", ({ input, value }) => sets.set(input, value));
  const observation = observe ? view.observe(() => undefined) : undefined;
  view.render(tree);
  return { area, view, sets, observation };
}

/**
 * Waits until `check` passes, as long as frames take to load while every package's tests run at once: longer than
 * `vi.waitFor`'s default second.
 */
async function waitFor(check: () => void): Promise<void> {
  await vi.waitFor(check, { timeout: 8000, interval: 50 });
}

/** Waits until `done` holds of what `observation` keeps. */
async function until(observation: Observation, done: (events: readonly ObservedEvent[]) => boolean): Promise<void> {
  await waitFor(() => {
    expect(done(observation.events)).toBe(true);
  });
}

type Method = (this: MessagePort, ...args: unknown[]) => unknown;

/**
 * Replaces the host's page's `MessagePort.prototype[name]` with what `wrap` makes of it, for this test: the returned
 * function, and the test's end, put it back.
 */
function patch(name: "postMessage" | "addEventListener", wrap: (original: Method) => Method): () => void {
  // `addEventListener` is `EventTarget`'s: the patch is `MessagePort`'s own, and goes again with the test.
  const own = Object.getOwnPropertyDescriptor(MessagePort.prototype, name);
  const original = Reflect.get(MessagePort.prototype, name) as Method;
  Object.defineProperty(MessagePort.prototype, name, { configurable: true, writable: true, value: wrap(original) });
  const restore = () => {
    if (own) Object.defineProperty(MessagePort.prototype, name, own);
    else Reflect.deleteProperty(MessagePort.prototype, name);
  };
  restores.push(restore);
  return restore;
}

/** The frame the host's page made for `component`, as it saw it. */
const frameOf = (events: readonly ObservedEvent[], component: string) => events.find((event) => event.type === "frame" && event.renders.kind === "use" && event.renders.component === component && event.by === undefined);

describe("the observer", () => {
  it("sees the whole tree in host mode: its own frames, and what each frame reports of its own, marked", async () => {
    const { observation } = open("host", nested, nestedTree);
    if (!observation) return;
    await until(observation, (events) => events.some((event) => event.type === "mounted" && event.component === "legend"));
    const { events } = observation;
    const map = frameOf(events, "map");
    const chart = frameOf(events, "chart");
    expect(map).toMatchObject({ level: "permissive", network: [] });
    expect(chart).toMatchObject({ level: "strict" });
    expect(events.some((event) => event.type === "frame" && event.renders.kind === "compile" && event.by === undefined)).toBe(true);
    const starsMounted = events.find((event) => event.type === "mounted" && event.component === "stars");
    expect(starsMounted?.frame).toBe("");
    expect(starsMounted && "by" in starsMounted).toBe(false);
    const mapId = map?.frame ?? "";
    // The map's frame reports the legend's frame it made, the messages on its port, and the components in each.
    expect(events.find((event) => event.type === "frame" && event.frame === `${mapId}/1`)).toMatchObject({ level: "strict", renders: { kind: "use", component: "legend" }, by: mapId });
    expect(events.find((event) => event.type === "mounted" && event.component === "map")).toMatchObject({ frame: mapId, by: mapId });
    expect(events.find((event) => event.type === "mounted" && event.component === "legend")).toMatchObject({ frame: `${mapId}/1`, by: mapId });
    const parts = events.filter((event) => event.type === "message" && event.way === "down" && (event.data as { type?: string }).type === "part");
    expect(parts.map(({ frame, by }) => [frame, by]).sort()).toEqual([[`${mapId}/1`, mapId], [chart?.frame, undefined], [mapId, undefined]].sort());
  });

  it("sees the page frame's components in frame mode, and where each fallback is", async () => {
    const { observation } = open("frame", nested, nestedTree);
    if (!observation) return;
    await until(observation, (events) => events.filter((event) => event.type === "fallback").length >= 2 && events.some((event) => event.type === "mounted"));
    const { events } = observation;
    const page = events.find((event) => event.type === "frame" && event.renders.kind === "page");
    expect(page).toMatchObject({ level: "strict" });
    expect(page && "by" in page).toBe(false);
    const id = page?.frame ?? "";
    expect(events.find((event) => event.type === "mounted")).toMatchObject({ component: "chart", frame: id, by: id });
    expect(events.flatMap((event) => (event.type === "fallback" ? [[event.component, event.reason, event.frame, event.by]] : [])).sort()).toEqual([
      ["map", "not-here", id, id],
      ["stars", "not-here", id, id],
    ]);
  });

  it("marks a forged report as the frame's, under its id, and acts on none of it (O6)", async () => {
    const forger = probing(
      "chart",
      { level: "strict" },
      `async () => {
        const post = MessagePort.prototype.postMessage;
        MessagePort.prototype.postMessage = function (message, transfer) {
          if (message && message.type === "observed") {
            message = { ...message, events: [...message.events,
              { type: "frame", frame: "", level: "permissive", renders: { kind: "page" }, network: ["*"], at: 1 },
              { type: "frame", frame: "7", level: "permissive", renders: { kind: "use", component: "anything" }, network: ["*"], at: 1 },
              { type: "gone", frame: "../1", why: "hung", at: 1 },
            ] };
          }
          return post.call(this, message, transfer);
        };
        return "patched";
      }`,
    );
    const { area, observation, sets } = open("host", [forger], treeOf([use("chart")], ["chart"]));
    if (!observation) return;
    await until(observation, (events) => events.some((event) => event.type === "frame" && event.frame.endsWith("/7")));
    const chart = frameOf(observation.events, "chart");
    const id = chart?.frame ?? "";
    expect(chart).toMatchObject({ level: "strict" });
    const forged = observation.events.filter((event) => event.by === id && event.type === "frame");
    expect(forged.map(({ frame }) => frame).sort()).toEqual([id, `${id}/7`]);
    expect(observation.events.some((event) => event.type === "gone")).toBe(false);
    expect(sets.get("chart")).toBe(JSON.stringify("patched"));
    expect(area.querySelectorAll("iframe")).toHaveLength(1);
  });

  it("drops a report no one asked for, and is turned on by nothing a frame sends (O4, O7)", async () => {
    const sent: unknown[] = [];
    patch("postMessage", (post) =>
      function (this: MessagePort, ...args: unknown[]) {
        sent.push(args[0]);
        return post.apply(this, args);
      },
    );
    const unasked = probing(
      "chart",
      { level: "strict" },
      `async () => {
        const port = await new Promise((resolve) => {
          const post = MessagePort.prototype.postMessage;
          MessagePort.prototype.postMessage = function (message, transfer) { resolve(this); return post.call(this, message, transfer); };
        });
        port.postMessage({ type: "observe", on: true });
        port.postMessage({ type: "observed", events: [{ type: "frame", frame: "5", level: "permissive", renders: { kind: "page" }, at: 1 }] });
        return "sent";
      }`,
    );
    const { view, sets } = open("host", [unasked], treeOf([use("chart")], ["chart"]), false);
    await waitFor(() => {
      expect(sets.get("chart")).toBe(JSON.stringify("sent"));
    });
    await pause(300);
    expect(sent.some((message) => (message as { type?: string }).type === "observe")).toBe(false);
    const observation = view.observe(() => undefined);
    await pause(500);
    expect(observation.events.some((event) => event.type === "frame" && event.frame.endsWith("/5"))).toBe(false);
    expect(sent.filter((message) => (message as { type?: string }).type === "observe")).toEqual([{ type: "observe", on: true }]);
  });

  it("records a `set` the host's page refused, of an input no use of the frame binds", async () => {
    const setter = probing(
      "chart",
      { level: "strict" },
      `async () => {
        const port = await new Promise((resolve) => {
          const post = MessagePort.prototype.postMessage;
          MessagePort.prototype.postMessage = function (message, transfer) { resolve(this); return post.call(this, message, transfer); };
        });
        port.postMessage({ type: "set", input: "secret", value: 1 });
        return "set";
      }`,
    );
    const { observation, sets } = open("host", [setter], treeOf([use("chart")], ["chart"]));
    if (!observation) return;
    await until(observation, (events) => events.some((event) => event.type === "refused"));
    const chart = frameOf(observation.events, "chart");
    const refused = observation.events.find((event) => event.type === "refused");
    expect(refused).toMatchObject({ frame: chart?.frame, input: "secret" });
    expect(refused && "by" in refused).toBe(false);
    expect(sets.has("secret")).toBe(false);
  });

  it("keeps the last 2,000 events of a frame that floods its reports, and the host's page's own (O2)", async () => {
    const flood = probing(
      "chart",
      { level: "strict" },
      `async () => {
        const port = await new Promise((resolve) => {
          const post = MessagePort.prototype.postMessage;
          MessagePort.prototype.postMessage = function (message, transfer) { if (message && message.type === "observed") resolve(this); return post.call(this, message, transfer); };
        });
        const events = Array.from({ length: 2000 }, (_, at) => ({ type: "dropped", frame: "", count: 0, at }));
        for (let round = 0; round < 10; round += 1) port.postMessage({ type: "observed", events });
        return "flooded";
      }`,
    );
    const { observation, sets } = open("host", [flood], treeOf([use("chart")], ["chart"]));
    if (!observation) return;
    await waitFor(() => {
      expect(sets.get("chart")).toBe(JSON.stringify("flooded"));
    });
    const id = () => frameOf(observation.events, "chart")?.frame ?? "";
    await waitFor(() => {
      expect(observation.dropped[id()] ?? 0).toBeGreaterThan(10_000);
    });
    expect(observation.events.filter((event) => event.by === id()).length).toBeLessThanOrEqual(2000);
    expect(frameOf(observation.events, "chart")).toBeDefined();
  });

  it("changes no message the sandbox sends or gets: the same with the observer as without (O3)", async () => {
    const record = async (observe: boolean) => {
      const seen: string[] = [];
      const counted = (data: unknown) => {
        const type = (data as { type?: unknown } | null)?.type;
        if (typeof type === "string" && !["ping", "pong", "size", "observe", "observed"].includes(type)) seen.push(type);
      };
      const restorePost = patch("postMessage", (post) =>
        function (this: MessagePort, ...args: unknown[]) {
          counted(args[0]);
          return post.apply(this, args);
        },
      );
      // What each port gets, once, however many listeners it has: the observer adds one of its own.
      const counting = new WeakSet<MessagePort>();
      const restoreListen = patch("addEventListener", (listen) =>
        function (this: MessagePort, ...args: unknown[]) {
          if (args[0] === "message" && !counting.has(this)) {
            counting.add(this);
            listen.call(this, "message", (event: MessageEvent) => {
              counted(event.data);
            });
          }
          return listen.apply(this, args);
        },
      );
      const { sets, view } = open("host", nested, nestedTree, observe);
      await waitFor(() => {
        expect(sets.size).toBe(2);
      });
      await pause(300);
      view.destroy();
      restorePost();
      restoreListen();
      return seen.sort();
    };
    const without = await record(false);
    const observed = await record(true);
    expect(without.length).toBeGreaterThan(10);
    expect(observed).toEqual(without);
  });
});
