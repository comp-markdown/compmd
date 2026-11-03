// One window's observer (docs/devtools.md, decisions 5 and 6): what it records once started, what it relays of its
// frames' reports, and a frame's batches.

import { afterEach, describe, expect, it, vi } from "vitest";
import { batches, Observer } from "./observer.ts";
import { settle } from "./protocol/channels.testing.ts";
import { SANDBOX_VERSION } from "./protocol/messages.ts";
import type { Observed } from "./protocol/observed.ts";
import { connectChild } from "./protocol/parent-end.ts";

afterEach(() => {
  vi.useRealTimers();
});

/** An observer whose events, with who reported them, are collected. */
function observed() {
  const got: { events: Observed[]; by: string | undefined }[] = [];
  const observer = new Observer((events, by) => got.push({ events, by }));
  const all = () => got.flatMap(({ events }) => events);
  return { observer, got, all };
}

/** A child frame, connected to `observer` as `id`, with its end of the port. */
function child(observer: Observer, id: string) {
  const { port1, port2: frame } = new MessageChannel();
  const end = connectChild(observer.port(id, port1), { level: "strict", binds: () => false });
  observer.connected(id, end);
  const toFrame: unknown[] = [];
  frame.addEventListener("message", ({ data }) => toFrame.push(data));
  frame.start();
  frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
  return { end, frame, toFrame };
}

describe("Observer", () => {
  it("records nothing until it is started, then the live frames and components first", () => {
    const { observer, all } = observed();
    const id = observer.frame("strict", { kind: "use", component: "chart" });
    const use = observer.mounted("stars", 1);
    observer.fallback("map", "not-here", "no");
    observer.refused(id, "secret");
    expect(all()).toEqual([]);
    observer.start();
    expect(all().map(({ type, frame }) => [type, frame])).toEqual([
      ["frame", id],
      ["mounted", ""],
    ]);
    observer.refused(id, "secret");
    observer.unmounted(use);
    observer.gone(id, "removed");
    expect(all().map(({ type }) => type)).toEqual(["frame", "mounted", "refused", "unmounted", "gone"]);
  });

  it("reports a frame gone once, and only a live one", () => {
    const { observer, all } = observed();
    observer.start();
    const id = observer.frame("strict", { kind: "page" });
    observer.gone(id, "hung");
    observer.gone(id, "removed");
    observer.gone("9", "removed");
    expect(all().map(({ type }) => type)).toEqual(["frame", "gone"]);
    expect(observer.live()).toEqual([]);
  });

  it("copies the messages on a frame's port both ways, but not pings, pongs or its own", async () => {
    const { observer, all } = observed();
    const id = observer.frame("strict", { kind: "use", component: "chart" });
    const { end, frame } = child(observer, id);
    observer.start();
    await settle();
    end.code([]);
    frame.postMessage({ type: "size", width: 1, height: 2 });
    frame.postMessage({ type: "pong", n: 1 });
    await settle();
    const messages = all().flatMap((event) => (event.type === "message" ? [[event.frame, event.way, (event.data as { type: string }).type]] : []));
    expect(messages).toEqual([
      [id, "up", "ready"],
      [id, "down", "level"],
      [id, "down", "code"],
      [id, "up", "size"],
    ]);
  });

  it("asks its frames to report once started, and to stop once stopped", async () => {
    const { observer } = observed();
    const { toFrame } = child(observer, observer.frame("strict", { kind: "page" }));
    observer.start();
    observer.stop();
    await settle();
    expect(toFrame).toEqual([{ type: "level", level: "strict" }, { type: "observe", on: true }, { type: "observe", on: false }]);
  });

  it("asks a frame connected while it records to report at once", async () => {
    const { observer } = observed();
    observer.start();
    const { toFrame } = child(observer, observer.frame("strict", { kind: "page" }));
    await settle();
    expect(toFrame).toContainEqual({ type: "observe", on: true });
  });

  it("relays a frame's report under its id, marked as its, with what isn't an event dropped", async () => {
    const { observer, got } = observed();
    observer.start();
    const id = observer.frame("permissive", { kind: "use", component: "map" });
    const { frame } = child(observer, id);
    await settle();
    frame.postMessage({
      type: "observed",
      events: [
        { type: "frame", frame: "1", level: "strict", renders: { kind: "use", component: "legend" }, at: 1 },
        { type: "mounted", frame: "1", use: 1, component: "legend", ms: 2, at: 2 },
        { type: "fallback", frame: "", component: "map", reason: "threw", message: "boom", at: 3 },
        { type: "frame", frame: "../1", level: "permissive", renders: { kind: "page" }, at: 4 },
        "not an event",
      ],
    });
    await settle();
    const reported = got.filter(({ by }) => by === id);
    expect(reported.flatMap(({ events }) => events.map(({ type, frame: where }) => [type, where]))).toEqual([
      ["frame", `${id}/1`],
      ["mounted", `${id}/1`],
      ["fallback", id],
    ]);
  });

  it("relays nothing of a frame while stopped, nor of an end that was replaced", async () => {
    const { observer, got } = observed();
    const id = observer.frame("strict", { kind: "page" });
    const first = child(observer, id);
    observer.start();
    const second = child(observer, id);
    await settle();
    const report = { type: "observed", events: [{ type: "dropped", frame: "", count: 1, at: 1 }] };
    first.frame.postMessage(report);
    await settle();
    expect(got.filter(({ by }) => by === id)).toEqual([]);
    second.frame.postMessage(report);
    await settle();
    expect(got.filter(({ by }) => by === id)).toHaveLength(1);
    observer.stop();
    second.frame.postMessage(report);
    await settle();
    expect(got.filter(({ by }) => by === id)).toHaveLength(1);
  });
});

describe("batches", () => {
  it("sends what was recorded every 100 ms, in one batch", () => {
    vi.useFakeTimers();
    const sent: Observed[][] = [];
    const { sink } = batches((events) => sent.push(events));
    sink([{ type: "dropped", frame: "", count: 1, at: 1 }]);
    sink([{ type: "dropped", frame: "", count: 2, at: 2 }]);
    expect(sent).toEqual([]);
    vi.advanceTimersByTime(100);
    expect(sent.map((batch) => batch.length)).toEqual([2]);
  });

  it("holds at most 2,000 events between batches, and says how many it dropped", () => {
    vi.useFakeTimers();
    const sent: Observed[][] = [];
    const { sink } = batches((events) => sent.push(events));
    sink(Array.from({ length: 2500 }, (_, at) => ({ type: "dropped" as const, frame: "", count: 0, at })));
    vi.advanceTimersByTime(100);
    expect(sent[0]).toHaveLength(2001);
    expect(sent[0]?.at(-1)).toMatchObject({ type: "dropped", frame: "", count: 500 });
  });

  it("drops what waits when cleared", () => {
    vi.useFakeTimers();
    const sent: Observed[][] = [];
    const { sink, clear } = batches((events) => sent.push(events));
    sink([{ type: "dropped", frame: "", count: 1, at: 1 }]);
    clear();
    vi.advanceTimersByTime(1000);
    expect(sent).toEqual([]);
  });
});
