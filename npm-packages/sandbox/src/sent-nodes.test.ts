import type { ExpandedNode, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { SentNodes } from "./sent-nodes.ts";

const word = (value: string): ExpandedNode => ({ type: "text", value });
/** The texts of `nodes`: the keys of this test. */
const words = (nodes: readonly ExpandedNode[]): string[] => nodes.flatMap((node) => (node.type === "text" ? [String(node.value)] : []));
const splice = (...nodes: ExpandedNode[]): Splice => ({ from: 0, to: 0, nodes, lines: 0, offset: 0 });

describe("SentNodes", () => {
  it("has the keys of the last tree sent, and not those of the tree before", () => {
    const sent = new SentNodes(words);
    sent.rendered([word("a"), word("b")]);
    expect(sent.has("a")).toBe(true);
    sent.rendered([word("c")]);
    expect(sent.has("a")).toBe(false);
    expect(sent.has("c")).toBe(true);
  });

  it("adds the keys of splices, those given as JSON text read only when asked", () => {
    const sent = new SentNodes(words);
    sent.rendered([word("a")]);
    sent.spliced([splice(word("b")), JSON.stringify(splice(word("c")))]);
    expect(sent.has("b")).toBe(true);
    expect(sent.has("c")).toBe(true);
    expect(sent.some((key) => key.startsWith("c"))).toBe(true);
    expect(sent.some((key) => key === "d")).toBe(false);
  });

  it("drops splices not yet read when a new tree comes", () => {
    const sent = new SentNodes(words);
    sent.spliced([JSON.stringify(splice(word("old")))]);
    sent.rendered([word("new")]);
    expect(sent.has("old")).toBe(false);
  });
});
