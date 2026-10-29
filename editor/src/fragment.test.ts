import { describe, expect, it } from "vitest";
import { directiveFor, findPassage, fragmentOf, parseFragment } from "./fragment.ts";

describe("parseFragment", () => {
  it("reads a text directive's parts, percent-decoded", () => {
    expect(parseFragment("#:~:text=price:%2012")).toEqual({ start: "price: 12" });
    expect(parseFragment(":~:text=title:-,Learn,CompMD,-%0Aauthor")).toEqual({ prefix: "title:", start: "Learn", end: "CompMD", suffix: "\nauthor" });
    expect(parseFragment("#top:~:text=a%2Db&text=other")).toEqual({ start: "a-b" });
  });

  it("is undefined for other links, and for directives that aren't one", () => {
    expect(parseFragment("#lesson-3")).toBeUndefined();
    expect(parseFragment("https://example.com")).toBeUndefined();
    expect(parseFragment("#:~:text=")).toBeUndefined();
    expect(parseFragment("#:~:text=a,b,c")).toBeUndefined();
    expect(parseFragment("#:~:text=%E0%A4%A")).toBeUndefined();
  });
});

describe("findPassage", () => {
  const text = "price: 12\nprice: 9\n\n```js\nconst a = 1;\nconst b = 2;\n```\n";

  it("finds the first passage, after its prefix and before its suffix, from its start to its end", () => {
    expect(findPassage(text, { start: "price:" })).toEqual({ start: 0, end: 6 });
    expect(findPassage(text, { prefix: "12", start: "price:" })).toEqual({ start: 10, end: 16 });
    expect(findPassage(text, { start: "price:", suffix: "9" })).toEqual({ start: 10, end: 16 });
    const fence = findPassage(text, { start: "```js", end: "```" });
    expect(fence && text.slice(fence.start, fence.end)).toBe("```js\nconst a = 1;\nconst b = 2;\n```");
  });

  it("is undefined when the text has no such passage", () => {
    expect(findPassage(text, { start: "cost" })).toBeUndefined();
    expect(findPassage(text, { start: "price:", end: "nowhere" })).toBeUndefined();
    expect(findPassage(text, { prefix: "nothing", start: "price:" })).toBeUndefined();
  });
});

describe("fragmentOf", () => {
  it("writes a directive with the characters its syntax uses encoded, and reads back the same", () => {
    const directive = { prefix: "a-b,", start: "c & d", end: "e", suffix: "f" };
    const fragment = fragmentOf(directive);
    expect(fragment).toBe(":~:text=a%2Db%2C-,c%20%26%20d,e,-f");
    expect(parseFragment(fragment)).toEqual(directive);
  });
});

describe("directiveFor", () => {
  it("names a short passage by itself, or with the words before it when the text has it earlier", () => {
    const text = "price: 12\nprice: 9\n";
    expect(directiveFor(text, { start: 0, end: 9 })).toEqual({ start: "price: 12" });
    expect(directiveFor(text, { start: 10, end: 15 })).toEqual({ prefix: "price: 12", start: "price" });
  });

  it("names a long passage by its first and last words", () => {
    const text = "Intro.\n\n```js\nconst a = 1;\nconst b = 2;\n```\n";
    const start = text.indexOf("```js");
    const end = text.lastIndexOf("```") + 3;
    const directive = directiveFor(text, { start, end });
    expect(directive).toEqual({ start: "```js", end: "```" });
    expect(directive && findPassage(text, directive)).toEqual({ start, end });
  });

  it("is undefined for an empty passage", () => {
    expect(directiveFor("a b", { start: 1, end: 2 })).toBeUndefined();
  });
});
