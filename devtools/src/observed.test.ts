// The new tabs' content, of what the sandbox's observer keeps (docs/devtools.md, decision 7, O6): the Sandbox tab's tree,
// the Console's history and the Messages tab's timeline.

import type { Diagnostic } from "@compmd/processor";
import type { ObservedEvent } from "@compmd/sandbox";
import { describe, expect, it } from "vitest";
import { consoleOf, EditorHistory } from "./console.ts";
import { parentOf, treeOf } from "./frames.ts";
import { timelineOf } from "./messages.ts";

/** The host-mode example of docs/devtools.md: a chart, a map with a legend in its slot, and the host's stars. */
const EXAMPLE: ObservedEvent[] = [
  { type: "frame", frame: "1", level: "strict", renders: { kind: "compile" }, at: 1 },
  { type: "frame", frame: "2", level: "strict", renders: { kind: "use", component: "chart" }, at: 2 },
  { type: "frame", frame: "3", level: "permissive", renders: { kind: "use", component: "map" }, network: ["https://tiles.example.com"], at: 3 },
  { type: "mounted", frame: "", use: 1, component: "stars", ms: 0.5, at: 4 },
  { type: "message", frame: "3", way: "down", data: { type: "part", part: {} }, at: 5 },
  { type: "mounted", frame: "3", use: 1, component: "map", ms: 3, at: 6, by: "3" },
  { type: "frame", frame: "3/1", level: "strict", renders: { kind: "use", component: "legend" }, at: 7, by: "3" },
  { type: "message", frame: "3/1", way: "down", data: { type: "part", part: {} }, at: 8, by: "3" },
  { type: "mounted", frame: "3/1", use: 1, component: "legend", ms: 1, at: 9, by: "3" },
  { type: "updated", frame: "3/1", use: 1, ms: 0.25, at: 10, by: "3" },
  { type: "message", frame: "2", way: "up", data: { type: "rendered", ms: 4 }, at: 11 },
  { type: "fallback", frame: "2", component: "chart", reason: "threw", message: "boom", at: 12, by: "2" },
];

describe("treeOf", () => {
  it("nests each frame under the window that made it, with its components, times and fallbacks", () => {
    const tree = treeOf(EXAMPLE);
    expect(tree).toMatchObject({ id: "", level: "host", uses: [{ component: "stars", ms: 0.5 }] });
    expect(tree.frames.map(({ id }) => id)).toEqual(["1", "2", "3"]);
    const [, chart, map] = tree.frames;
    expect(chart).toMatchObject({ renders: { component: "chart" }, rendered: 4, fallbacks: [{ component: "chart", by: "2" }] });
    expect(map).toMatchObject({ level: "permissive", network: ["https://tiles.example.com"], uses: [{ component: "map", by: "3" }] });
    expect(map?.frames).toMatchObject([{ id: "3/1", level: "strict", by: "3", uses: [{ component: "legend", updated: 0.25, updates: 1 }] }]);
  });

  it("keeps a frame's report of a frame the host's page saw apart, as its claim (O6)", () => {
    const tree = treeOf([...EXAMPLE, { type: "frame", frame: "2", level: "permissive", renders: { kind: "page" }, network: ["*"], at: 13, by: "2" }]);
    const chart = tree.frames.find(({ id }) => id === "2");
    expect(chart).toMatchObject({ level: "strict", renders: { kind: "use", component: "chart" } });
    expect(chart).not.toHaveProperty("by");
    expect(chart?.claims).toEqual([expect.objectContaining({ level: "permissive", by: "2" })]);
  });

  it("leaves out a frame that went, with the frames in it", () => {
    const tree = treeOf([...EXAMPLE, { type: "gone", frame: "3", why: "hung", at: 13 }]);
    expect(tree.frames.map(({ id }) => id)).toEqual(["1", "2"]);
  });

  it("leaves out a component that unmounted", () => {
    const tree = treeOf([...EXAMPLE, { type: "unmounted", frame: "", use: 1, at: 13 }]);
    expect(tree.uses).toEqual([]);
  });

  it("knows each frame's parent from its id", () => {
    expect(parentOf("3/1")).toBe("3");
    expect(parentOf("3")).toBe("");
  });
});

const diagnostic = (code: string): Diagnostic => ({ code, severity: "error", message: "no such component", range: { start: { line: 1, column: 1 } }, stage: "resolve" });

describe("the console", () => {
  it("lists each change's diagnostics when they change, and each problem once", () => {
    const history = new EditorHistory();
    history.note([], [], 1);
    history.note([diagnostic("F002")], [], 2);
    history.note([diagnostic("F002")], ["@kit/x: no version 2"], 3);
    history.note([diagnostic("F002")], ["@kit/x: no version 2"], 4);
    history.note([], [], 5);
    expect(history.entries.map((entry) => [entry.kind, entry.at])).toEqual([
      ["diagnostics", 2],
      ["problem", 3],
      ["diagnostics", 5],
    ]);
  });

  it("puts the sandbox's fallbacks, refused sets and stopped frames among them, in time, each marked as its reporter's", () => {
    const history = new EditorHistory();
    history.note([diagnostic("F002")], [], 11.5);
    const events: ObservedEvent[] = [
      ...EXAMPLE,
      { type: "refused", frame: "2", input: "secret", at: 13 },
      { type: "gone", frame: "3", why: "hung", at: 14 },
      { type: "gone", frame: "1", why: "removed", at: 15 },
    ];
    expect(consoleOf(history.entries, events)).toEqual([
      { kind: "diagnostics", at: 11.5, diagnostics: [diagnostic("F002")] },
      { kind: "fallback", at: 12, frame: "2", component: "chart", reason: "threw", code: "W118", message: "boom", by: "2" },
      { kind: "refused", at: 13, frame: "2", input: "secret" },
      { kind: "stopped", at: 14, frame: "3", why: "hung" },
    ]);
  });
});

describe("timelineOf", () => {
  it("puts each message in its frame's lane, the lanes in the tree's order", () => {
    const { lanes, rows } = timelineOf([...EXAMPLE, { type: "message", frame: "1", way: "up", data: "not a message", at: 13 }]);
    expect(lanes).toEqual(["1", "2", "3", "3/1"]);
    expect(rows.map(({ frame, way, type, by }) => [frame, way, type, by])).toEqual([
      ["3", "down", "part", undefined],
      ["3/1", "down", "part", "3"],
      ["2", "up", "rendered", undefined],
      ["1", "up", "?", undefined],
    ]);
  });
});
