import { afterEach, describe, expect, it, vi } from "vitest";
import { keep, recall } from "./stored.ts";

afterEach(() => {
  vi.restoreAllMocks();
  window.localStorage.clear();
});

describe("what the editor remembers", () => {
  it("keeps and recalls a preference", () => {
    expect(recall("a")).toBeUndefined();
    keep("a", "b");
    expect(recall("a")).toBe("b");
  });

  it("forgets quietly when storage refuses, or isn't there", () => {
    const blocked = () => {
      throw new Error("blocked");
    };
    vi.spyOn(window, "localStorage", "get").mockReturnValue({ getItem: blocked, setItem: blocked } as unknown as Storage);
    expect(() => {
      keep("a", "b");
    }).not.toThrow();
    expect(recall("a")).toBeUndefined();
    vi.spyOn(window, "localStorage", "get").mockImplementation(blocked);
    expect(recall("a")).toBeUndefined();
  });
});
