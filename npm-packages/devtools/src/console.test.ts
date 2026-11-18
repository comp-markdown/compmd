import { describe, expect, it } from "vitest";
import { ExpressionHistory, markedAt, withEvaluations, type ConsoleEntry } from "./console.ts";

describe("markedAt", () => {
  it("marks the column under its line, on a line of its own", () => {
    expect(markedAt("md.doc.tem", 1, 8)).toBe("md.doc.tem\n       ^");
    expect(markedAt("[1,\n  1 / 0]", 2, 5)).toBe("[1,\n  1 / 0]\n    ^");
    expect(markedAt("x", 1, 1)).toBe("x\n^");
  });
});

describe("withEvaluations", () => {
  it("puts the evaluations among the console's lines, by when each happened", () => {
    const entries: ConsoleEntry[] = [
      { kind: "problem", at: 1, message: "first" },
      { kind: "problem", at: 3, message: "third" },
    ];
    const lines = withEvaluations(entries, [{ id: 1, at: 2, expression: "md.doc" }]);
    expect(lines.map((line) => line.kind)).toEqual(["problem", "evaluation", "problem"]);
  });
});

describe("ExpressionHistory", () => {
  it("walks back with previous and forward with next, to what was being written", () => {
    const history = new ExpressionHistory();
    expect(history.previous("draft")).toBeUndefined();
    history.add("a");
    history.add("b");
    history.add("b");
    expect(history.previous("half")).toBe("b");
    expect(history.previous("b")).toBe("a");
    expect(history.previous("a")).toBeUndefined();
    expect(history.next()).toBe("b");
    expect(history.next()).toBe("half");
    expect(history.next()).toBeUndefined();
    history.add("c");
    expect(history.previous("")).toBe("c");
  });
});
