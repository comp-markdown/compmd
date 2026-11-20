import type { CodeLines, ExpandedNode } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { forSent, oneBlockATask, sentBlocks } from "./frames-highlight.ts";

const code = (value: string, lang: string): ExpandedNode => ({ type: "code", value, lang });
const plain = (text: string): CodeLines => text.split("\n").map((line) => [{ text: line }]);

describe("oneBlockATask", () => {
  it("asks the highlighter one block at a time, in the order the frames ask, a task apart", async () => {
    const asked: string[] = [];
    let running = 0;
    let most = 0;
    const highlight = oneBlockATask(async (text) => {
      running += 1;
      most = Math.max(most, running);
      asked.push(text);
      await Promise.resolve();
      running -= 1;
      return plain(text);
    });
    const answers = await Promise.all(["a", "b", "c"].map((text) => highlight(text, "ts")));
    expect(asked).toEqual(["a", "b", "c"]);
    expect(most).toBe(1);
    expect(answers).toEqual([plain("a"), plain("b"), plain("c")]);
  });

  it("answers none for a language it doesn't cover, or when it fails, and goes on with the next", async () => {
    const highlight = oneBlockATask((text, language) => {
      if (language === "boom") throw new Error("no grammar");
      return language === "ts" ? plain(text) : undefined;
    });
    expect(await Promise.all([highlight("a", "boom"), highlight("b", "cobol"), highlight("c", "ts")])).toEqual([undefined, undefined, plain("c")]);
  });
});

describe("forSent", () => {
  it("asks only for a block of what was sent, by its text and language", async () => {
    const blocks = sentBlocks();
    blocks.rendered([code("let a = 1;", "ts")]);
    const asked: string[] = [];
    const highlight = forSent(blocks, (text, language) => {
      asked.push(`${language}:${text}`);
      return Promise.resolve(plain(text));
    });
    expect(await highlight("let a = 1;", "ts")).toEqual(plain("let a = 1;"));
    expect(await highlight("let a = 1;", "js")).toBeUndefined();
    expect(await highlight("secret text", "ts")).toBeUndefined();
    expect(asked).toEqual(["ts:let a = 1;"]);
  });
});
