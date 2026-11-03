import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount, { rowsOf } from "./csv.ts";

describe("rowsOf", () => {
  it("splits fields at commas, but not inside quotes", () => {
    expect(rowsOf('city,visits\n"Kyoto, Japan",12\n')).toEqual([
      ["city", "visits"],
      ["Kyoto, Japan", "12"],
    ]);
  });
});

describe("csv", () => {
  it("draws the block's rows under its header, captioned with its alt, and follows new params", () => {
    const shown = mountForTest(mount, { params: { code: "city,visits\nKyoto,12", header: true }, fallback: "Visits by city" });
    expect(shown.target.querySelector("caption")?.textContent).toBe("Visits by city");
    expect([...shown.target.querySelectorAll("th")].map((cell) => cell.textContent)).toEqual(["city", "visits"]);
    shown.update({ code: "Kyoto,12\nOsaka,7", header: false });
    expect(shown.target.querySelectorAll("th")).toHaveLength(0);
    expect(shown.target.querySelectorAll("tr")).toHaveLength(2);
    shown.destroy();
    expect(shown.target.children).toHaveLength(0);
  });
});
