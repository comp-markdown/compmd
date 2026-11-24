// The compile frame's parent end (docs/sandbox-protocol.md), `connectCompiler`, against the frame's end.

import { describe, expect, it } from "vitest";
import { connectCompiler, serveParent } from "./index.ts";

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
