import { expect, it } from "vitest";
import { currentOf } from "./current.ts";

const tops: Record<string, number> = { intro: -400, terms: 60, syntax: 900 };
const top = (id: string) => tops[id] ?? 0;

it("is the last heading at or above the line", () => {
  expect(currentOf(["intro", "terms", "syntax"], top, 80)).toBe("terms");
  expect(currentOf(["intro", "terms", "syntax"], top, 60)).toBe("terms");
  expect(currentOf(["intro", "terms", "syntax"], top, 59)).toBe("intro");
});

it("is the first while none has been reached, and nothing without headings", () => {
  expect(currentOf(["terms", "syntax"], top, 0)).toBe("terms");
  expect(currentOf([], top, 0)).toBeUndefined();
});
