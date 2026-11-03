import { describe, expect, it } from "vitest";
import { snippet } from "./search.ts";

describe("snippet", () => {
  it("shows the text around the first word found, each word found marked", () => {
    const text = "A document loads its data before it renders, and fetches remote data while it is read, data by data.";
    expect(snippet(text, ["fetch"], 40)).toEqual([
      { text: "…and ", marked: false },
      { text: "fetch", marked: true },
      { text: "es remote data while it is…", marked: false },
    ]);
  });

  it("marks a word whatever its case, and shows the start when none is found", () => {
    expect(snippet("Data, then DATA.", ["data"])).toEqual([
      { text: "Data", marked: true },
      { text: ", then ", marked: false },
      { text: "DATA", marked: true },
      { text: ".", marked: false },
    ]);
    expect(snippet("Nothing here.", ["absent"])).toEqual([{ text: "Nothing here.", marked: false }]);
    expect(snippet("Nothing here.", [])).toEqual([{ text: "Nothing here.", marked: false }]);
  });

  it("reads a word with a regular expression's characters as text", () => {
    expect(snippet("Use a+b here.", ["a+b"])).toEqual([
      { text: "Use ", marked: false },
      { text: "a+b", marked: true },
      { text: " here.", marked: false },
    ]);
  });
});
