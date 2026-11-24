// The compile frame's parent end (docs/sandbox-protocol.md), `connectCompiler`, against the frame's end.

import { describe, expect, it, vi } from "vitest";
import { COMPILE_MS } from "./compiler.ts";
import { connectCompiler, serveParent } from "./index.ts";
import { SANDBOX_VERSION, type Port } from "./messages.ts";

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

  it("gives two components of one name, a document's and an example's, each its own code (S3)", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const compiler = connectCompiler(parent);
    // The frame answers the second request first, as a compile that takes longer would.
    serveParent(frame, {
      level: () => undefined,
      compile: ({ code }) =>
        new Promise((resolve) => {
          setTimeout(() => {
            resolve({ code: code.toUpperCase() });
          }, code === "first" ? 50 : 0);
        }),
    });
    const [first, second] = await Promise.all([compiler.compile({ name: "card", language: "jsx", code: "first" }), compiler.compile({ name: "card", language: "jsx", code: "second" })]);
    expect([first.code, second.code]).toEqual(["FIRST", "SECOND"]);
    compiler.close();
  });

  it("drops an answer to no request it made", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const compiler = connectCompiler(parent);
    frame.onmessage = ({ data }: MessageEvent<{ type: string; id?: number; code?: string }>) => {
      if (data.type !== "compile" || data.id === undefined) return;
      // An answer with a made-up id first, then the real one.
      frame.postMessage({ type: "compiled", id: data.id + 100, name: "card", code: "not this" });
      frame.postMessage({ type: "compiled", id: data.id, name: "card", code: "this" });
    };
    frame.postMessage({ type: "ready", sandbox: SANDBOX_VERSION });
    await expect(compiler.compile({ name: "card", language: "jsx", code: "x" })).resolves.toEqual({ code: "this" });
    compiler.close();
    frame.close();
  });

  it("fails a compile the frame never answers, after its timeout", async () => {
    vi.useFakeTimers();
    try {
      // A port the test answers itself, at once: the frame says `ready`, then never answers.
      let listener: (event: { data: unknown }) => void = () => undefined;
      const port: Port = {
        postMessage: () => undefined,
        addEventListener: (_type, added) => (listener = added),
        removeEventListener: () => undefined,
      };
      const compiler = connectCompiler(port);
      listener({ data: { type: "ready", sandbox: SANDBOX_VERSION } });
      const never = compiler.compile({ name: "card", language: "jsx", code: "x" });
      const failed = expect(never).rejects.toThrow(`didn't answer for \`card\` within ${String(COMPILE_MS / 1000)} s`);
      await vi.advanceTimersByTimeAsync(COMPILE_MS + 1000);
      await failed;
      compiler.close();
    } finally {
      vi.useRealTimers();
    }
  });

  it("fails every request when the frame is of another version, saying so (B8)", async () => {
    const { port1: parent, port2: frame } = new MessageChannel();
    const compiler = connectCompiler(parent, { ready: { timeout: 60_000, url: "https://frames.example/strict.html" } });
    const waiting = compiler.compile({ name: "a", language: "jsx", code: "x" });
    frame.postMessage({ type: "ready", sandbox: 1 });
    await expect(waiting).rejects.toThrow("the compile frame is of protocol version 1");
    await expect(compiler.compile({ name: "b", language: "jsx", code: "y" })).rejects.toThrow("of protocol version 1");
    compiler.close();
    frame.close();
  });

  it("fails every request when told to: the frame can't answer them", async () => {
    const { port1: parent } = new MessageChannel();
    const compiler = connectCompiler(parent);
    const waiting = compiler.compile({ name: "a", language: "jsx", code: "x" });
    compiler.fail(new Error("the compile frame navigated"));
    await expect(waiting).rejects.toThrow("navigated");
    await expect(compiler.compile({ name: "b", language: "jsx", code: "y" })).rejects.toThrow("navigated");
    compiler.close();
  });
});
