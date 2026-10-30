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

/** A frame's port and its parent's, with every message the parent got. */
function channel() {
  const { port1: parent, port2: frame } = new MessageChannel();
  const got: unknown[] = [];
  parent.addEventListener("message", ({ data }) => got.push(data));
  parent.start();
  return { parent, frame: frame as Port, got };
}

/** Handlers that record what the frame was told. */
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

describe("the frame's end", () => {
  it("says it is ready, and acts on nothing before its level", async () => {
    const { parent, frame, got } = channel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "code", components: [] });
    parent.postMessage({ type: "ping", n: 1 });
    await settle();
    expect(got).toEqual([{ type: "ready", sandbox: SANDBOX_VERSION }]);
    expect(calls).toEqual([]);
  });

  it("takes its level once, then the rest in order", async () => {
    const { parent, frame, got } = channel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "level", level: "permissive", policy: "connect-src *" });
    parent.postMessage({ type: "code", components: [{ name: "chart", level: "strict", binding: "plain", code: "export default () => {}" }] });
    parent.postMessage({ type: "part", part: { node: chart, inputs: {} } });
    parent.postMessage({ type: "styles", urls: ["/docs.css"] });
    parent.postMessage({ type: "ping", n: 7 });
    parent.postMessage({ type: "unknown" });
    parent.postMessage("not a message");
    await settle();
    expect(calls.map(([name]) => name)).toEqual(["level", "code", "part", "styles"]);
    expect(calls[0]).toEqual(["level", { level: "strict", policy: undefined }]);
    expect(got.slice(1)).toEqual([{ type: "loaded" }, { type: "pong", n: 7 }]);
  });

  it("takes `host` only as a permissive frame", async () => {
    for (const level of ["strict", "permissive"] as const) {
      const { parent, frame } = channel();
      const { calls, handlers } = recording();
      serveParent(frame, handlers);
      parent.postMessage({ type: "level", level });
      parent.postMessage({ type: "host" }, [new MessageChannel().port1]);
      parent.postMessage({ type: "host" });
      await settle();
      expect(calls.filter(([name]) => name === "host"), level).toEqual(level === "permissive" ? [["host", true]] : []);
    }
  });

  it("tells its parent what its components do", async () => {
    const { frame, got } = channel();
    const end = serveParent(frame, { level: () => undefined });
    end.set("region", "eu");
    end.error("threw", "boom", "chart");
    end.error("not-here", "map can't be in a strict frame");
    end.link("https://example.com");
    end.rendered(3);
    end.size(320, 200);
    end.close();
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

  it("compiles, as the compile frame", async () => {
    const { parent, frame, got } = channel();
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

describe("the parent's end", () => {
  it("waits for the frame, gives it its level first, then what was sent meanwhile", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const got: unknown[] = [];
    frame.addEventListener("message", ({ data }) => got.push(data));
    frame.start();
    const child = connectChild(parent, { level: "permissive", policy: "connect-src https://tiles.example.com", binds: () => false });
    child.code([]);
    child.part({ node: chart, inputs: {} });
    child.styles(["/docs.css"]);
    child.send({ type: "theme", tokens: {} });
    await settle();
    expect(got).toEqual([]);
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(got.map((message) => (message as { type: string }).type)).toEqual(["level", "code", "part", "styles", "theme"]);
    expect(got[0]).toEqual({ type: "level", level: "permissive", policy: "connect-src https://tiles.example.com" });
    child.code([]);
    await settle();
    expect(got).toHaveLength(6);
  });

  it("gives a frame of another version nothing", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const got: unknown[] = [];
    frame.addEventListener("message", ({ data }) => got.push(data));
    frame.start();
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

  it("passes on only the inputs the frame's uses bind, and what else the frame says", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const bound = boundInputs([chart]);
    const child = connectChild(parent, { level: "strict", binds: (input) => bound.has(input) });
    const events: unknown[] = [];
    for (const type of ["set", "error", "link", "loaded", "rendered", "size"] as const) child.on(type, (event) => events.push([type, event]));
    frame.postMessage({ type: "set", input: "region", value: "eu" });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    frame.postMessage({ type: "set", input: "region", value: "us" });
    frame.postMessage({ type: "set", input: "secret", value: "x" });
    frame.postMessage({ type: "error", reason: "threw", component: "chart", message: "boom" });
    frame.postMessage({ type: "link", url: "https://example.com" });
    frame.postMessage({ type: "loaded" });
    frame.postMessage({ type: "rendered", ms: 2 });
    frame.postMessage({ type: "size", width: 10, height: 20 });
    frame.postMessage({ type: "compiled", name: "x", code: "" });
    await settle();
    expect(events).toEqual([
      ["set", { input: "region", value: "us" }],
      ["error", { reason: "threw", component: "chart", message: "boom" }],
      ["link", { url: "https://example.com" }],
      ["loaded", {}],
      ["rendered", { ms: 2 }],
      ["size", { width: 10, height: 20 }],
    ]);
  });

  it("gives `host` only to a permissive frame, after its level", async () => {
    expect(() => connectChild(new MessageChannel().port1, { level: "strict", binds: () => false, host: new MessageChannel().port1 })).toThrow("only a permissive frame");
    const { port1: parent, port2: frame } = new MessageChannel();
    const got: [unknown, number][] = [];
    frame.addEventListener("message", ({ data, ports }) => got.push([data, ports.length]));
    frame.start();
    connectChild(parent, { level: "permissive", binds: () => false, host: new MessageChannel().port1 });
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await settle();
    expect(got).toEqual([
      [{ type: "level", level: "permissive" }, 0],
      [{ type: "host" }, 1],
    ]);
  });

  it("stops a frame that stops answering, and keeps one that answers", async () => {
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
    for (let second = 0; second < 5; second += 1) {
      vi.advanceTimersByTime(1000);
      await settle();
    }
    expect(failed).toEqual([]);
    answering = false;
    for (let second = 0; second < 5; second += 1) {
      vi.advanceTimersByTime(1000);
      await settle();
    }
    expect(failed).toEqual([{ reason: "hung" }]);
  });
});

describe("the compile frame's parent end", () => {
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
  it("finds the inputs bound in slots and nested components", () => {
    const tabs: ExpandedComponent = {
      type: "componentContainer",
      name: "tabs",
      params: [{ key: "open", breakpoint: null, value: 0, input: "tab" }],
      children: [chart],
      slots: [{ type: "componentSlot", name: "footer", children: [{ ...chart, params: [{ key: "x", breakpoint: null, value: 1, input: "year" }] }] }],
    };
    expect([...boundInputs([tabs])].sort()).toEqual(["region", "tab", "year"]);
  });
});
