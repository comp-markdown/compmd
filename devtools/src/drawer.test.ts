import { describe, expect, it } from "vitest";
import { DRAWER_MIN, drawerHeight, VIEWS_MIN } from "./drawer.ts";

describe("drawerHeight", () => {
  it("takes the height wanted, leaving the views their least, and never less than its own least", () => {
    expect(drawerHeight(200, 600)).toBe(200);
    expect(drawerHeight(560, 600)).toBe(600 - VIEWS_MIN);
    expect(drawerHeight(20, 600)).toBe(DRAWER_MIN);
    expect(drawerHeight(300, 150)).toBe(DRAWER_MIN);
  });
});
