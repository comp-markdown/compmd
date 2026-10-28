import { readFileSync } from "node:fs";
import { join } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { changelog, completeExample, completeJson, longBody } from "../bench/requests.ts";
import { initSync, process, sources } from "./index.ts";

/** `value` without positions, as the conformance case compares it. */
function withoutPositions(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(withoutPositions);
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value)
      .filter(([key]) => key !== "position" && key !== "closePosition")
      .map(([key, field]) => [key, withoutPositions(field)]),
  );
}

beforeAll(() => {
  initSync({ module: readFileSync(join(import.meta.dirname, "../pkg/compmd_wasm_bg.wasm")) });
});

describe("process", () => {
  it("expands the spec's complete example as its conformance case expects, time zones from Intl", () => {
    const processed = process(completeExample());
    expect(withoutPositions(processed.expanded)).toEqual(withoutPositions(completeJson("expanded.json")));
    expect(processed.diagnostics).toEqual([]);
    expect(processed.parse.type).toBe("root");
    expect(processed.parse.children[0]?.type).toBe("yaml");
  });

  it("expands the bench's other documents without diagnostics", () => {
    for (const request of [changelog(), longBody()]) {
      const processed = process(request);
      expect(processed.diagnostics).toEqual([]);
      expect(processed.expanded.children.length).toBeGreaterThan(10);
    }
  });

  it("gives origins when asked", () => {
    const processed = process({ path: "a.comp.md", files: { "a.comp.md": "Hello\n" }, now: "2026-10-22T09:00:00Z", origins: true });
    expect(processed.expanded.children[0]?.origin).toEqual({
      start: { line: 1, column: 1, offset: 0 },
      end: { line: 1, column: 6, offset: 5 },
      occurrence: 0,
    });
  });

  it("reports diagnostics with their stage", () => {
    const processed = process({ path: "a.comp.md", files: { "a.comp.md": ":: nothing[Text]\n" }, now: "2026-10-22T09:00:00Z" });
    expect(processed.diagnostics).toMatchObject([{ code: "F002", severity: "error", range: { start: { line: 1, column: 4 } }, stage: "resolve" }]);
  });

  it("throws on a request it can't process", () => {
    expect(() => process({ path: "a.comp.md", files: {}, now: "2026-10-22T09:00:00Z" })).toThrow("`a.comp.md` isn't one of the request's files");
    expect(() => process({ path: "a.comp.md", files: { "a.comp.md": "" }, now: "today" })).toThrow("`today` isn't an RFC 3339 time");
  });
});

describe("sources", () => {
  it("lists what the host fetches", () => {
    expect(sources(completeExample())).toEqual({
      remote: ["https://status.rayplay.dev/today.json?region=all"],
      host: ["tasks?project=rayplay"],
    });
  });
});
