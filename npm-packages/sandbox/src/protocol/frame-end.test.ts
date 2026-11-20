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
    scheme: (scheme) => calls.push(["scheme", scheme]),
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

  it("takes the page's colour scheme, and reads anything but dark as light", async () => {
    const { parent, frame } = frameChannel();
    const { calls, handlers } = recording();
    serveParent(frame, handlers);
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "scheme", scheme: "dark" });
    parent.postMessage({ type: "scheme", scheme: "sepia" });
    await settle();
    expect(calls.slice(1)).toEqual([
      ["scheme", "dark"],
      ["scheme", "light"],
    ]);
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

  it("starts and stops observing as its parent asks, after its level, and reports what it observed", async () => {
    const { parent, frame, got } = frameChannel();
    const asked: boolean[] = [];
    const end = serveParent(frame, { level: () => undefined, observe: (on) => asked.push(on) });
    parent.postMessage({ type: "observe", on: true });
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "observe", on: true });
    parent.postMessage({ type: "observe", on: false });
    end.observed([{ type: "dropped", frame: "", count: 1, at: 1 }]);
    await settle();
    expect(asked).toEqual([true, false]);
    expect(got.slice(1)).toEqual([{ type: "observed", events: [{ type: "dropped", frame: "", count: 1, at: 1 }] }]);
  });

  it("measures again when its parent says the box changed, at the box's width, after its level, and reports the size", async () => {
    const { parent, frame, got } = frameChannel();
    const widths: number[] = [];
    const end = serveParent(frame, { level: () => undefined, box: (width) => widths.push(width) });
    parent.postMessage({ type: "box", width: 100 });
    parent.postMessage({ type: "level", level: "strict" });
    parent.postMessage({ type: "box", width: 464 });
    // Strict code may have written these: no width, or none that is a size.
    parent.postMessage({ type: "box" });
    parent.postMessage({ type: "box", width: -1 });
    parent.postMessage({ type: "box", width: "wide" });
    end.size(300, 150.5);
    await settle();
    expect(widths).toEqual([464]);
    expect(got.slice(1)).toEqual([{ type: "size", width: 300, height: 150.5 }]);
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

  it("asks for an image once while it waits, and gives each asker the file its parent answered, or none", async () => {
    const { parent, frame, got } = frameChannel();
    const end = serveParent(frame, { level: () => undefined });
    parent.postMessage({ type: "level", level: "strict" });
    await settle();
    const first = end.image("./logo.png");
    const second = end.image("./logo.png");
    const missing = end.image("./missing.png");
    await settle();
    expect(got.slice(1)).toEqual([
      { type: "image", url: "./logo.png" },
      { type: "image", url: "./missing.png" },
    ]);
    const file = new Blob(["x"], { type: "image/png" });
    parent.postMessage({ type: "fetched", url: "./logo.png", image: file });
    parent.postMessage({ type: "fetched", url: "./missing.png", image: "not a Blob" });
    // Posting copies the file: the same type and bytes, not the same object.
    expect(await first).toEqual(file);
    expect(await second).toEqual(file);
    expect(await missing).toBeUndefined();
    const refused = end.image("https://photos.example.com/a.png");
    await settle();
    parent.postMessage({ type: "fetched", url: "https://photos.example.com/a.png", refused: "https://photos.example.com" });
    expect(await refused).toEqual({ refused: "https://photos.example.com" });
  });

  it("asks its parent for a code block's tokens by an id of its own, and takes each answer by its id, in any order", async () => {
    const { parent, frame, got } = frameChannel();
    const end = serveParent(frame, { level: () => undefined });
    parent.postMessage({ type: "level", level: "strict" });
    await settle();
    const ts = end.highlight("let a", "ts");
    const unknown = end.highlight("x", "cobol");
    const forged = end.highlight("y", "ts");
    await settle();
    expect(got.slice(1)).toEqual([
      { type: "highlight", id: 1, code: "let a", language: "ts" },
      { type: "highlight", id: 2, code: "x", language: "cobol" },
      { type: "highlight", id: 3, code: "y", language: "ts" },
    ]);
    const lines = [[{ text: "let ", color: { light: "#d73a49", dark: "#f97583" } }, { text: "a" }]];
    parent.postMessage({ type: "highlighted", id: 2 });
    parent.postMessage({ type: "highlighted", id: 1, lines });
    parent.postMessage({ type: "highlighted", id: 3, lines: "not lines" });
    expect(await ts).toEqual(lines);
    expect(await unknown).toBeUndefined();
    expect(await forged).toBeUndefined();
  });

  it("keeps a block's text when its parent doesn't answer within the level's timeout", async () => {
    const { parent, frame } = frameChannel();
    const end = serveParent(frame, { level: () => undefined });
    parent.postMessage({ type: "level", level: "strict", timeout: 10 });
    await settle();
    const waited = end.highlight("let a", "ts");
    await new Promise((resolve) => setTimeout(resolve, 30));
    // A late answer finds nothing waiting.
    parent.postMessage({ type: "highlighted", id: 1, lines: [[{ text: "let a" }]] });
    expect(await waited).toBeUndefined();
  });
});
