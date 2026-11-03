import { describe, expect, it } from "vitest";
import { clamp, DEFAULT, keyed, load, MAX, MIN, save } from "./split.ts";

describe("the split", () => {
  it("keeps each open side at least a fifth of the width", () => {
    expect(clamp(0.05)).toBe(MIN);
    expect(clamp(0.95)).toBe(MAX);
    expect(clamp(0.5)).toBe(0.5);
  });

  it("moves with the arrow keys, folds with Home and End, and opens with Enter", () => {
    const split = { at: 0.5, folded: undefined };
    expect(keyed(split, "ArrowLeft")?.at).toBeCloseTo(0.48);
    expect(keyed({ at: MAX, folded: undefined }, "ArrowRight")?.at).toBe(MAX);
    expect(keyed(split, "Home")).toEqual({ at: 0.5, folded: "text" });
    expect(keyed(split, "End")).toEqual({ at: 0.5, folded: "views" });
    expect(keyed({ at: 0.5, folded: "views" }, "Enter")).toEqual({ at: 0.5, folded: undefined });
    expect(keyed({ at: 0.5, folded: "text" }, "ArrowRight")).toEqual({ at: 0.52, folded: undefined });
    expect(keyed(split, "a")).toBeUndefined();
  });

  it("is remembered, and falls back to the default when storage has nothing usable or refuses", () => {
    const stored = new Map<string, string>();
    const storage = {
      getItem: (key: string) => stored.get(key) ?? null,
      setItem: (key: string, value: string) => {
        stored.set(key, value);
      },
    };
    save(storage, { at: 0.3, folded: "views" });
    expect(load(storage)).toEqual({ at: 0.3, folded: "views" });
    expect(load(undefined)).toEqual(DEFAULT);
    expect(load({ getItem: () => "{not json" })).toEqual(DEFAULT);
    expect(load({ getItem: () => '{"at": 0.01, "folded": "odd"}' })).toEqual({ at: MIN, folded: undefined });
    const refusing = {
      setItem: () => {
        throw new Error("blocked");
      },
    };
    expect(() => {
      save(refusing, DEFAULT);
    }).not.toThrow();
  });
});
