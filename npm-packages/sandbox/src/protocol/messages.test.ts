import { describe, expect, it } from "vitest";
import { bindingOf, diagnosticOf, FROM_FRAME, hasCode, isMessage, TO_FRAME, type FromFrame, type ToFrame } from "./messages.ts";

describe("bindingOf", () => {
  it("mounts JSX and TSX through React's binding, Svelte through Svelte's, JavaScript and TypeScript as plain", () => {
    expect([bindingOf("jsx"), bindingOf("tsx"), bindingOf("svelte"), bindingOf("js"), bindingOf("ts")]).toEqual(["react", "react", "svelte", "plain", "plain"]);
  });
});

describe("hasCode", () => {
  it("is true of a component with text, a Blob or a module's URL", () => {
    expect(hasCode({ name: "chart", level: "strict", binding: "react", code: "" })).toBe(true);
    expect(hasCode({ name: "diagram", level: "strict", binding: "plain", code: new Blob([]), place: "@kit/a@1.0.0/a.js" })).toBe(true);
    expect(hasCode({ name: "map", level: "strict", module: { url: "/a.js", integrity: "sha384-x" } })).toBe(true);
  });

  it("is false of one sent by name and level only, which a frame can't hold", () => {
    expect(hasCode({ name: "stars", level: "trusted" })).toBe(false);
  });
});

describe("isMessage", () => {
  it("takes a message of the types asked for", () => {
    expect(isMessage<ToFrame>({ type: "ping", n: 1 }, TO_FRAME)).toBe(true);
    expect(isMessage<FromFrame>({ type: "pong", n: 1 }, FROM_FRAME)).toBe(true);
  });

  it("refuses one of the other way, an unknown type, and what isn't an object", () => {
    expect(isMessage<ToFrame>({ type: "pong", n: 1 }, TO_FRAME)).toBe(false);
    expect(isMessage<ToFrame>({ type: "other" }, TO_FRAME)).toBe(false);
    expect(isMessage<ToFrame>("ping", TO_FRAME)).toBe(false);
    expect(isMessage<ToFrame>(null, TO_FRAME)).toBe(false);
  });
});

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
