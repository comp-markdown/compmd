// The protocol's ends (docs/sandbox-protocol.md), each rule a test: what each end does with what it is sent, and
// what it refuses. The frame's end is `serveParent`, its parent's `connectChild`, the compile frame's `connectCompiler`.

import type { ExpandedComponent } from "@compmd/render";
import { afterEach, describe, expect, it, vi } from "vitest";
import { boundInputs, connectChild, connectCompiler, SANDBOX_VERSION, serveParent, type FrameHandlers, type Port } from "./index.ts";

/** Lets the messages posted so far arrive. */
const settle = () =>
  new Promise<void>((resolve) => {
    setImmediate(() => {
      setImmediate(resolve);
    });
  });

/** A frame's port, and its parent's end of it, which records every message the frame sends. */
function frameChannel() {
  const { port1: parent, port2: frame } = new MessageChannel();
  const got: unknown[] = [];
  parent.addEventListener("message", ({ data }) => got.push(data));
  parent.start();
  return { parent, frame: frame as Port, got };
}

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

/** The types of `messages`. */
const types = (messages: unknown[]) => messages.map((message) => (message as { type: string }).type);

/** Frame handlers that record what the frame was told. */
function recording() {
  const calls: [string, unknown][] = [];
  const handlers: FrameHandlers = {
    level: (level, policy) => calls.push(["level", { level, policy }]),
    code: (components) => {
      calls.push(["code", components]);
      return Promise.resolve();
    },
    part: (part) => calls.push(["part", part]),
    styles: (urls) => calls.push(["styles", urls]),
    host: (port) => calls.push(["host", port instanceof MessagePort]),
  };
  return { calls, handlers };
}

const chart: ExpandedComponent = {
  type: "componentLeaf",
  name: "chart",
  params: [
    { key: "rows", breakpoint: null, value: [] },
    { key: "selected", breakpoint: null, value: "eu", input: "region" },
  ],
};

afterEach(() => {
  vi.useRealTimers();
});

describe("the frame's end (serveParent)", () => {
  it("says it is ready, with the protocol's version, as it starts", async () => {
    const { frame, got } = frameChannel();
    serveParent(frame, recording().handlers);
    await settle();
    expect(got).toEqual([{ type: "ready", sandbox: SANDBOX_VERSION }]);
  });

  it("acts on nothing before its level, not even a ping: its policy isn't narrowed yet", async () => {
    const { parent, frame, got } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "code", components: [] });
    parent.postMessage({ type: "ping", n: 1 });
    await settle();
    expect(calls).toEqual([]);
    expect(types(got)).toEqual(["ready"]);
  });

  it("takes its level once: a second level, which could widen a permissive policy, is ignored", async () => {
    const { parent, frame } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "level", level: "permissive", policy: "connect-src *" });
    await settle();
    expect(calls).toEqual([["level", { level: "strict", policy: undefined }]]);
  });

  it("takes code, a part and stylesheets after its level, in the order sent, and says when the code is loaded", async () => {
    const { parent, frame, got } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "code", components: [{ name: "chart", level: "strict", binding: "plain", code: "export default () => {}" }] });
    parent.postMessage({ type: "part", part: { node: chart, inputs: {} } });
    parent.postMessage({ type: "styles", urls: ["/docs.css"] });
    await settle();
    expect(calls.map(([name]) => name)).toEqual(["level", "code", "part", "styles"]);
    expect(types(got)).toEqual(["ready", "loaded"]);
  });

  it("answers each ping with its number", async () => {
    const { parent, frame, got } = frameChannel();
    serveParent(frame, recording().handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "ping", n: 7 });
    await settle();
    expect(got.slice(1)).toEqual([{ type: "pong", n: 7 }]);
  });

  it("ignores what isn't one of the protocol's messages", async () => {
    const { parent, frame, got } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "unknown" });
    parent.postMessage("not a message");
    parent.postMessage(null);
    await settle();
    expect(calls.map(([name]) => name)).toEqual(["level"]);
    expect(types(got)).toEqual(["ready"]);
  });

  it("takes `host` as a permissive frame, with its port", async () => {
    const { parent, frame } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "permissive" });
    parent.postMessage({ type: "host" }, [new MessageChannel().port1]);
    await settle();
    expect(calls.filter(([name]) => name === "host")).toEqual([["host", true]]);
  });

  it("refuses `host` as a strict frame, and without a port", async () => {
    const { parent, frame } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "host" }, [new MessageChannel().port1]);
    parent.postMessage({ type: "host" });
    await settle();
    expect(calls.filter(([name]) => name === "host")).toEqual([]);
  });

  it("tells its parent what its components do: set, errors, links, renders, its size", async () => {
    const { frame, got } = frameChannel();
    const end = serveParent(frame, { level: () => undefined });
    end.set("region", "eu");
    end.error("threw", "boom", "chart");
    end.error("not-here", "map can't be in a strict frame");
    end.link("https://example.com");
    end.rendered(3);
    end.size(320, 200);
    await settle();
    expect(got.slice(1)).toEqual([
      { type: "set", input: "region", value: "eu" },
      { type: "error", reason: "threw", message: "boom", component: "chart" },
      { type: "error", reason: "not-here", message: "map can't be in a strict frame" },
      { type: "link", url: "https://example.com" },
      { type: "rendered", ms: 3 },
      { type: "size", width: 320, height: 200 },
    ]);
  });

  it("acts on nothing once closed", async () => {
    const { parent, frame } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers).close();
    parent.postMessage({ type: "level", level: "strict" });
    await settle();
    expect(calls).toEqual([]);
  });

  it("compiles, as the compile frame, and says what didn't compile", async () => {
    const { parent, frame, got } = frameChannel();
    serveParent(frame, {
      level: () => undefined,
      compile: ({ code }) => (code === "bad" ? Promise.reject(new Error("unexpected token")) : Promise.resolve({ code: `compiled ${code}`, css: "p{}" })),
    });
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "compile", name: "a", language: "jsx", code: "good" });
    parent.postMessage({ type: "compile", name: "b", language: "svelte", code: "bad" });
    await settle();
    expect(got.slice(1)).toEqual([
      { type: "compiled", name: "a", code: "compiled good", css: "p{}" },
      { type: "failed", name: "b", message: "unexpected token" },
    ]);
  });
});

describe("the parent's end (connectChild)", () => {
  it("sends nothing before the frame is ready, then its level first and what was sent meanwhile, in order", async () => {
    const { parent, frame, got } = parentChannel();
    const child = connectChild(parent, { level: "permissive", policy: "connect-src https://tiles.example.com", binds: () => false });
    child.code([]);
    child.part({ node: chart, inputs: {} });
    child.styles(["/docs.css"]);
    child.send({ type: "theme", tokens: {} });
    await settle();
    expect(got).toEqual([]);
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(types(got)).toEqual(["level", "code", "part", "styles", "theme"]);
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

describe("the compile frame's parent end (connectCompiler)", () => {
  it("compiles once the frame is ready, and says what didn't compile", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const compiler = connectCompiler(parent);
    serveParent(frame, {
      level: () => undefined,
      compile: ({ code }) => (code === "bad" ? Promise.reject(new Error("unexpected token")) : Promise.resolve({ code: code.toUpperCase() })),
    });
    await expect(compiler.compile({ name: "a", language: "jsx", code: "good" })).resolves.toEqual({ code: "GOOD" });
    await expect(compiler.compile({ name: "b", language: "jsx", code: "bad" })).rejects.toThrow("unexpected token");
    compiler.close();
  });
});

describe("boundInputs", () => {
  it("finds the inputs bound in a use, its slots and the components nested in them", () => {
    const tabs: ExpandedComponent = {
      type: "componentContainer",
      name: "tabs",
      params: [{ key: "open", breakpoint: null, value: 0, input: "tab" }],
      children: [chart],
      slots: [{ type: "componentSlot", name: "footer", children: [{ ...chart, params: [{ key: "x", breakpoint: null, value: 1, input: "year" }] }] }],
    };
    expect([...boundInputs([tabs])].sort()).toEqual(["region", "tab", "year"]);
  });

  it("is empty for uses that bind no input", () => {
    expect([...boundInputs([{ type: "componentLeaf", name: "legend", params: [{ key: "title", breakpoint: null, value: "Key" }] }])]).toEqual([]);
    expect([...boundInputs([])]).toEqual([]);
  });
});
