// @vitest-environment happy-dom
import type { CodeHighlighter } from "@compmd/render";
import { describe, expect, it, vi } from "vitest";
import { framesHighlight } from "./from-frames.ts";

/** A highlighter that gives each line as one token. */
const lines: CodeHighlighter = (code) => code.split("\n").map((text) => [{ text }]);

describe("framesHighlight", () => {
  it("makes one highlighter for every view of the page that loads it from one place", async () => {
    const load = vi.fn(() => Promise.resolve(lines));
    const [one, two] = [framesHighlight("https://app.example/compmd/one/", load), framesHighlight("https://app.example/compmd/one/", load)];
    expect(await Promise.all([one("a", "js"), two("b", "js"), one("c", "ts")])).toEqual([[[{ text: "a" }]], [[{ text: "b" }]], [[{ text: "c" }]]]);
    expect(load.mock.calls).toEqual([["https://app.example/compmd/one/highlight.js"]]);
  });

  it("makes one for each place it is loaded from", async () => {
    const load = vi.fn(() => Promise.resolve(lines));
    await framesHighlight("https://app.example/compmd/two/", load)("a", "js");
    await framesHighlight("https://app.example/compmd/three/", load)("a", "js");
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("loads again for the next block after a load that failed", async () => {
    const load = vi.fn<(url: string) => Promise<CodeHighlighter>>().mockRejectedValueOnce(new Error("offline")).mockResolvedValue(lines);
    const highlight = framesHighlight("https://app.example/compmd/four/", load);
    await expect(highlight("a", "js")).rejects.toThrow("offline");
    expect(await framesHighlight("https://app.example/compmd/four/", load)("a", "js")).toEqual([[{ text: "a" }]]);
    expect(load).toHaveBeenCalledTimes(2);
  });
});
