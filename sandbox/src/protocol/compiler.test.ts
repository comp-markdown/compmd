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
});
