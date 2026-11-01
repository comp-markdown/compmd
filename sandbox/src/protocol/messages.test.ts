import { describe, expect, it } from "vitest";
import { diagnosticOf } from "./messages.ts";

describe("diagnosticOf", () => {
  it("gives W118 for code that threw and W121 for code that didn't load", () => {
    expect(diagnosticOf("threw")).toBe("W118");
    expect(diagnosticOf("failed")).toBe("W121");
  });

  it("gives none for what the processor reports or the sandbox's levels decide", () => {
    expect(diagnosticOf("missing")).toBeUndefined();
    expect(diagnosticOf("not-here")).toBeUndefined();
  });
});
