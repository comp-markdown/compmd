import { describe, expect, it } from "vitest";
// @ts-expect-error: a plain module of wasm-bindgen's, without types.
import { zoneOffset } from "./zones.js";

const offset = zoneOffset as (zone: string, seconds: number) => number | undefined;
const SUMMER = Date.parse("2026-07-01T12:00:00Z") / 1000;
const WINTER = Date.parse("2026-12-01T12:00:00Z") / 1000;

describe("zoneOffset", () => {
  it("gives a zone's offset at a time, in seconds east of UTC", () => {
    expect(offset("UTC", SUMMER)).toBe(0);
    expect(offset("Europe/Berlin", SUMMER)).toBe(7200);
    expect(offset("Europe/Berlin", WINTER)).toBe(3600);
    expect(offset("America/New_York", WINTER)).toBe(-18000);
    expect(offset("Asia/Kolkata", SUMMER)).toBe(19800);
  });

  it("knows no zone Intl doesn't", () => {
    expect(offset("Mars/Olympus_Mons", SUMMER)).toBeUndefined();
  });
});
