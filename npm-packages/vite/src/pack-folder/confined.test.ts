import { describe, expect, it } from "vitest";
import { isPackPath, joinedInPack } from "./confined.ts";

describe("isPackPath", () => {
  it("takes `/`-separated names, and refuses what could leave the pack", () => {
    expect(isPackPath("dist/chart.js")).toBe(true);
    for (const path of ["", "./a.js", "../a.js", "/a.js", "a//b.js", "a\\b.js", "C:/a.js", "a\0b"]) expect(isPackPath(path), path).toBe(false);
  });
});

describe("joinedInPack", () => {
  it("resolves a relative path against the folder of the file it is written in", () => {
    expect(joinedInPack("index.comp.md", "./parts/card.comp.md")).toBe("parts/card.comp.md");
    expect(joinedInPack("parts/cards.comp.md", "../icons/star.svg")).toBe("icons/star.svg");
    expect(joinedInPack("a/b.comp.md", "./x/../y.csv")).toBe("a/y.csv");
  });

  it("refuses a path that leaves the pack, names its folder, starts at a root or has a Windows name", () => {
    for (const relative of ["../x.csv", "./", "/etc/passwd", "./..\\..\\secrets.txt", "./c:/x.jsx", "./a\0.csv"]) expect(joinedInPack("index.comp.md", relative), relative).toBeUndefined();
  });
});
