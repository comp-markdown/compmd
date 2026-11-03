// The parent's end of the protocol (docs/sandbox-protocol.md), `connectChild`, each rule a test: what it sends its
// frame, and what it passes on or drops of what the frame sends.

import { afterEach, describe, expect, it, vi } from "vitest";
import { chart, settle, types } from "./channels.testing.ts";
import { boundInputs, connectChild, SANDBOX_VERSION, type Port } from "./index.ts";

/** A parent's port, and the frame's end of it, which records every message the parent sends, with its ports. */
function parentChannel() {
  const { port1: parent, port2: frame } = new MessageChannel();
  const got: unknown[] = [];
  const ports: number[] = [];
  frame.addEventListener("message", ({ data, ports: sent }) => {
    got.push(data);
    ports.push(sent.length);
  });
  frame.start();
  return { parent: parent as Port, frame, got, ports };
}

afterEach(() => {
  vi.useRealTimers();
});

describe("the parent's end (connectChild)", () => {
  it("sends nothing before the frame is ready, then its level first and what was sent meanwhile, in order", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "permissive", policy: "connect-src https://tiles.example.com", binds: () => false });
    child.code([]);
    child.part({ node: chart, inputs: {} });
    child.styles(["/docs.css"]);
    child.scheme("dark");
    child.send({ type: "theme", tokens: {} });
    await settle();
    expect(got).toEqual([]);
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(types(got)).toEqual(["level", "code", "part", "styles", "scheme", "theme"]);
    expect(got[0]).toEqual({ type: "level", level: "permissive", policy: "connect-src https://tiles.example.com" });
  });

  it("sends at once once the frame is ready", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => false });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    child.code([]);
    await settle();
    expect(types(got)).toEqual(["level", "code"]);
  });

  it("gives its level once, however often the frame says it is ready", async () => {
    const { parent, frame, got } = parentChannel();
    connectChild(parent, { level: "strict", binds: () => false });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(types(got)).toEqual(["level"]);
  });

  it("gives a frame of another version nothing, and says it failed", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => true });
    const failed: unknown[] = [];
    child.on("failed", (failure) => failed.push(failure));
    child.code([]);
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION + 1 });
    await settle();
    child.code([]);
    await settle();
    expect(got).toEqual([]);
    expect(failed).toEqual([{ reason: "version", version: SANDBOX_VERSION + 1 }]);
  });

  it("passes on a `set` of an input one of the frame's uses binds", async () => {
    const { parent, frame } = parentChannel();
    const bound = boundInputs([chart]);
    const child = connectChild(parent, { level: "strict", binds: (input) => bound.has(input) });
    const sets: unknown[] = [];
    child.on("set", (event) => sets.push(event));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    frame.postMessage({ type: "set", input: "region", value: "us" });
    await settle();
    expect(sets).toEqual([{ input: "region", value: "us" }]);
  });

  it("drops a `set` of an input no use binds: strict code may have written it", async () => {
    const { parent, frame } = parentChannel();
    const bound = boundInputs([chart]);
    const child = connectChild(parent, { level: "strict", binds: (input) => bound.has(input) });
    const sets: unknown[] = [];
    child.on("set", (event) => sets.push(event));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    frame.postMessage({ type: "set", input: "secret", value: "x" });
    await settle();
    expect(sets).toEqual([]);
  });

  it("drops what the frame sends before it is ready", async () => {
    const { parent, frame } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => true });
    const sets: unknown[] = [];
    child.on("set", (event) => sets.push(event));
    frame.postMessage({ type: "set", input: "region", value: "eu" });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(sets).toEqual([]);
  });

  it("passes on the frame's errors, links, loads, renders and size", async () => {
    const { parent, frame } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => false });
    const events: unknown[] = [];
    for (const type of ["error", "link", "loaded", "rendered", "size"] as const) child.on(type, (event) => events.push([type, event]));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    frame.postMessage({ type: "error", reason: "threw", component: "chart", message: "boom" });
    frame.postMessage({ type: "link", url: "https://example.com" });
    frame.postMessage({ type: "loaded" });
    frame.postMessage({ type: "rendered", ms: 2 });
    frame.postMessage({ type: "size", width: 10, height: 20 });
    frame.postMessage({ type: "compiled", name: "x", code: "" });
    await settle();
    expect(events).toEqual([
      ["error", { reason: "threw", component: "chart", message: "boom" }],
      ["link", { url: "https://example.com" }],
      ["loaded", {}],
      ["rendered", { ms: 2 }],
      ["size", { width: 10, height: 20 }],
    ]);
  });

  it("gives `host` to a permissive frame, after its level, with the port", async () => {
    const { parent, frame, got, ports } = parentChannel();
    connectChild(parent, { level: "permissive", binds: () => false, host: new MessageChannel().port1 });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(got).toEqual([{ type: "level", level: "permissive" }, { type: "host" }]);
    expect(ports).toEqual([0, 1]);
  });

  it("refuses to give `host` to a strict frame", () => {
    expect(() => connectChild(new MessageChannel().port1, { level: "strict", binds: () => false, host: new MessageChannel().port1 })).toThrow("only a permissive frame is given host");
  });

  it("stops a frame that stops answering its pings, and keeps one that answers", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval", "Date"] });
    const { port1: parent, port2: frame } = new MessageChannel();
    let answering = true;
    frame.addEventListener("message", ({ data }) => {
      const message = data as { type: string; n: number };
      if (message.type === "ping" && answering) frame.postMessage({ type: "pong", n: message.n });
    });
    frame.start();
    const child = connectChild(parent, { level: "strict", binds: () => false, ping: { every: 1000, timeout: 3000 } });
    const failed: unknown[] = [];
    child.on("failed", (failure) => failed.push(failure));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    const seconds = async (count: number) => {
      for (let second = 0; second < count; second += 1) {
        vi.advanceTimersByTime(1000);
        await settle();
      }
    };
    await seconds(5);
    expect(failed).toEqual([]);
    answering = false;
    await seconds(5);
    expect(failed).toEqual([{ reason: "hung" }]);
  });

  it("pings no frame it wasn't asked to", async () => {
    vi.useFakeTimers({ toFake: ["setInterval", "clearInterval", "Date"] });
    const { parent, frame, got } = parentChannel();
    connectChild(parent, { level: "strict", binds: () => false });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    vi.advanceTimersByTime(10_000);
    await settle();
    expect(types(got)).toEqual(["level"]);
  });

  it("takes a frame's report only while it asks for one (docs/devtools.md, O7)", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => false });
    const reports: unknown[] = [];
    child.on("observed", ({ events }) => reports.push(events));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    const report = { type: "observed", events: [{ type: "dropped", frame: "", count: 1, at: 1 }] };
    frame.postMessage(report);
    await settle();
    expect(reports).toEqual([]);
    child.observe(true);
    child.observe(true);
    frame.postMessage(report);
    frame.postMessage({ type: "observed", events: "not a list" });
    await settle();
    expect(reports).toEqual([report.events]);
    child.observe(false);
    frame.postMessage(report);
    await settle();
    expect(reports).toHaveLength(1);
    expect(types(got)).toEqual(["level", "observe", "observe"]);
  });

  it("takes at most 2,000 events of one report", async () => {
    const { parent, frame } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => false });
    const reports: unknown[][] = [];
    child.on("observed", ({ events }) => reports.push(events));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    child.observe(true);
    frame.postMessage({ type: "observed", events: Array.from({ length: 2500 }, () => ({})) });
    await settle();
    expect(reports[0]).toHaveLength(2000);
  });

  it("sends and passes on nothing once closed", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "strict", binds: () => true });
    const sets: unknown[] = [];
    child.on("set", (event) => sets.push(event));
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    child.close();
    child.code([]);
    frame.postMessage({ type: "set", input: "region", value: "eu" });
    await settle();
    expect(types(got)).toEqual(["level"]);
    expect(sets).toEqual([]);
  });
});
