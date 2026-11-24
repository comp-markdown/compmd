import { describe, expect, it } from "vitest";
import { localDay } from "./local-day.ts";

describe("localDay", () => {
  it("gives the day of this machine's calendar, late in the evening and early in the morning", () => {
    expect(localDay(new Date(2026, 10, 23, 23, 30))).toBe("2026-11-23");
    expect(localDay(new Date(2026, 0, 1, 0, 15))).toBe("2026-01-01");
  });
});
