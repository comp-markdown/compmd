// What a frame may report (docs/devtools.md, O6): each event rebuilt from its fields, large values kept by size, and
// ids that nest under the frame that reported them.

import { describe, expect, it } from "vitest";
import { copyOf, observedOf, within } from "./observed.ts";

describe("copyOf", () => {
  it("keeps small values as they are", () => {
    const value = { type: "set", input: "region", value: ["eu", 2, true, null] };
    expect(copyOf(value)).toEqual(value);
  });

  it("keeps long text by its length and start", () => {
    const code = "x".repeat(5000);
    expect(copyOf({ code })).toEqual({ code: { $text: 5000, start: "x".repeat(2000) } });
  });

  it("keeps the first 200 items of a long list, and of an object's keys", () => {
    const list = copyOf(Array.from({ length: 250 }, (_, index) => index)) as unknown[];
    expect(list).toHaveLength(201);
    expect(list.at(-1)).toEqual({ $more: 50 });
    const keys = copyOf(Object.fromEntries(Array.from({ length: 205 }, (_, index) => [`k${String(index)}`, index]))) as Record<string, unknown>;
    expect(keys.$more).toBe(5);
  });

  it("keeps files, bytes and ports by what they are", () => {
    expect(copyOf({ code: new Blob(["abc"], { type: "text/javascript" }), bytes: new Uint8Array(4), port: new MessageChannel().port1 })).toEqual({
      code: { $blob: 3, mime: "text/javascript" },
      bytes: { $bytes: 4 },
      port: { $port: true },
    });
  });

  it("stops at a depth, and at a number of values", () => {
    let deep: unknown = "end";
    for (let depth = 0; depth < 40; depth += 1) deep = [deep];
    expect(JSON.stringify(copyOf(deep))).toContain('{"$more":1}');
    const wide = Array.from({ length: 200 }, () => Array.from({ length: 200 }, () => 1));
    expect(JSON.stringify(copyOf(wide))).toContain('{"$more":1}');
  });
});

describe("observedOf", () => {
  it("takes each kind of event", () => {
    const events = [
      { type: "frame", frame: "1", level: "permissive", renders: { kind: "use", component: "map" }, network: ["https://tiles.example.com"], at: 1 },
      { type: "gone", frame: "1/2", why: "hung", at: 2 },
      { type: "message", frame: "1", way: "up", data: { type: "size", width: 1, height: 2 }, at: 3 },
      { type: "mounted", frame: "", use: 1, component: "legend", ms: 0.5, at: 4 },
      { type: "updated", frame: "", use: 1, ms: 0.1, at: 5 },
      { type: "unmounted", frame: "", use: 1, at: 6 },
      { type: "fallback", frame: "", component: "legend", reason: "threw", message: "boom", at: 7 },
      { type: "dropped", frame: "", count: 3, at: 8 },
    ];
    expect(events.map(observedOf)).toEqual(events);
  });

  it("rebuilds an event from its own fields only", () => {
    expect(observedOf({ type: "gone", frame: "1", why: "removed", at: 1, extra: "<img src=x onerror=alert(1)>" })).toEqual({ type: "gone", frame: "1", why: "removed", at: 1 });
  });

  it("keeps a message's large values by size, and a long fallback message cut", () => {
    expect(observedOf({ type: "message", frame: "1", way: "down", data: { code: "y".repeat(3000) }, at: 1 })).toEqual({ type: "message", frame: "1", way: "down", data: { code: { $text: 3000, start: "y".repeat(2000) } }, at: 1 });
    const fallback = observedOf({ type: "fallback", frame: "", component: "c", reason: "threw", message: "z".repeat(3000), at: 1 }) as { message: string };
    expect(fallback.message).toHaveLength(2001);
  });

  it.each([
    ["a value that isn't an object", "frame"],
    ["an unknown type", { type: "other", frame: "", at: 1 }],
    ["a frame id that isn't a path of numbers", { type: "gone", frame: "../1", why: "hung", at: 1 }],
    ["a frame id that starts with a slash", { type: "gone", frame: "/1", why: "hung", at: 1 }],
    ["a time that isn't a number", { type: "gone", frame: "1", why: "hung", at: "now" }],
    ["a trusted frame", { type: "frame", frame: "1", level: "trusted", renders: { kind: "page" }, at: 1 }],
    ["a frame that renders something unknown", { type: "frame", frame: "1", level: "strict", renders: { kind: "other" }, at: 1 }],
    ["a reason no fallback has", { type: "fallback", frame: "", component: "c", reason: "other", message: "", at: 1 }],
    ["a way that isn't up or down", { type: "message", frame: "1", way: "sideways", data: {}, at: 1 }],
    ["a negative time to mount", { type: "mounted", frame: "", use: 1, component: "c", ms: -1, at: 1 }],
  ])("drops %s", (_rule, event) => {
    expect(observedOf(event)).toBeUndefined();
  });
});

describe("within", () => {
  it("nests a frame's ids under it, its own as its id", () => {
    expect(within("2", "")).toBe("2");
    expect(within("2", "1")).toBe("2/1");
    expect(within("2", "1/3")).toBe("2/1/3");
  });
});
