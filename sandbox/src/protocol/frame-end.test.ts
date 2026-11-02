// The frame's end of the protocol (docs/sandbox-protocol.md), `serveParent`, each rule a test: what it does with what
// its parent sends, and what it refuses.

import { describe, expect, it } from "vitest";
import { chart, settle, types } from "./channels.testing.ts";
import { SANDBOX_VERSION, serveParent, type FrameHandlers, type Port } from "./index.ts";

/** A frame's port, and its parent's end of it, which records every message the frame sends. */
function frameChannel() {
  const { port1: parent, port2: frame } = new MessageChannel();
  const got: unknown[] = [];
  parent.addEventListener("message", ({ data }) => got.push(data));
  parent.start();
  return { parent, frame: frame as Port, got };
}

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
