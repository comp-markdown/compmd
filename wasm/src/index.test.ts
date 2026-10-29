import { readFileSync } from "node:fs";
import { join } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { changelog, completeExample, completeJson, longBody } from "../bench/requests.ts";
import { componentsOf, frontmatterOf, initSync, OpenDocument, OpenDocumentJson, plainHtml, process, sources, type Request } from "./index.ts";

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

describe("frontmatterOf", () => {
  it("gives the document's fields, settings, data and inputs", () => {
    const frontmatter = frontmatterOf(completeExample());
    expect(frontmatter.doc).toMatchObject({ title: "RayPlay 1.4" });
    expect(frontmatter.settings).toMatchObject({ lang: "en", tz: "Europe/Berlin" });
    expect(frontmatter.data.map((data) => data.name).length).toBeGreaterThan(0);
    expect(Object.keys(frontmatter.inputs)).toContain("region");
  });
});

describe("componentsOf", () => {
  it("groups the components the document can name, with its uses", () => {
    const { groups, unknown } = componentsOf(completeExample());
    const named = groups.flatMap((group) => group.components.filter((c) => c.uses.length > 0).map((c) => [c.name, c.kind]));
    expect(named).toContainEqual(["chart", "jsx"]);
    expect(groups.at(-1)).toMatchObject({ provided: true });
    expect(unknown).toEqual([]);
  });
});

describe("plainHtml", () => {
  it("shows the text as a plain GFM renderer does", () => {
    expect(plainHtml(":badge[beta]{tone=info} ~~x~~\n")).toBe("<p>:badge[beta]{tone=info} <del>x</del></p>\n");
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

describe("OpenDocument", () => {
  /** An edit as an editor gives it: UTF-16 `[start, end)` of the text became `text`. */
  type Edit = [start: number, end: number, text: string];

  /**
   * `request` opened, then edited with each of `edits`: after each, the tree and diagnostics it keeps are those a new
   * `process` of the same text gives.
   */
  function checkEdits(request: Request, edits: Edit[]): OpenDocument {
    const document = new OpenDocument(request);
    let text = request.files?.[request.path] ?? "";
    for (const [start, end, inserted] of edits) {
      document.edit(start, end, inserted);
      text = text.slice(0, start) + inserted + text.slice(end);
      const fresh = process({ ...request, files: { ...request.files, [request.path]: text } });
      expect(document.expanded, `after ${JSON.stringify([start, end, inserted])}`).toEqual(fresh.expanded);
      expect(document.diagnostics).toEqual(fresh.diagnostics);
    }
    return document;
  }

  const now = "2026-10-22T09:00:00Z";
  const request = (text: string, origins = true): Request => ({ path: "a.comp.md", files: { "a.comp.md": text }, now, origins });
  const text = "😀 Intro\n\n# Head\n\nOne 👍🏽 two.\n\n:: nope[N]\n\nLast.\n";
  const at = (part: string) => text.indexOf(part);

  it("keeps the tree a new process gives, through edits in UTF-16 positions", () => {
    for (const origins of [true, false]) {
      checkEdits(request(text, origins), [
        [at(" Intro") + 6, at(" Intro") + 6, "!"], // after `Intro`, behind an emoji
        [at("two"), at("two") + 3, "three"], // after a thumb with a skin tone
        [at("# Head"), at("# Head"), "Pasted.\n\n## Sub\n\n"], // blocks above the heading
        [0, 2, ""], // the emoji
      ]);
    }
  });

  it("follows the bench's long body", () => {
    const body = longBody();
    const notes = body.files?.[body.path] ?? "";
    const middle = notes.indexOf("\n\n", notes.length / 2) + 2;
    checkEdits({ ...body, origins: true }, [
      [middle, middle, "x"],
      [middle, middle + 1, ""],
      [middle, middle, "New paragraph.\n\n"],
    ]);
  });

  it("gives what changed", () => {
    const document = new OpenDocument(request(text));
    const before = document.expanded.children.length;
    const last = document.expanded.children.at(-1)?.origin?.start.offset ?? 0;
    // A paragraph above `Last.`: it comes, and `Last.` moves down two lines and eight code points.
    const changes = document.edit(at("Last"), at("Last"), "Added.\n\n");
    expect(changes.splice).toMatchObject({ from: before - 1, to: before - 1, lines: 2, offset: 8 });
    expect(changes.splice.nodes).toHaveLength(1);
    expect(document.expanded.children).toHaveLength(before + 1);
    expect(document.expanded.children.at(-1)?.origin?.start.offset).toBe(last + 8);
    expect(changes.diagnostics.map((d) => d.code)).toEqual(["F002"]);
    // In code points: the emoji and the thumb with its skin tone before it are six UTF-16 units and three code points.
    expect(changes.splice.edit).toEqual({ start: at("Last") - 3, end: at("Last") - 3, text: "Added.\n\n" });
    document.free();
  });

  it("gives each change with its splice as JSON text, as OpenDocumentJson", () => {
    const document = new OpenDocumentJson(request(text));
    const opened = document.changes();
    expect(JSON.parse(opened.splice)).toMatchObject({ from: 0, to: 0 });
    expect(JSON.parse(opened.splice)).not.toHaveProperty("edit");
    const changes = document.edit(at("Last"), at("Last"), "Added.\n\n");
    expect(JSON.parse(changes.splice)).toMatchObject({ lines: 2, offset: 8, edit: { text: "Added.\n\n" } });
    expect(changes.diagnostics.map((d) => d.code)).toEqual(["F002"]);
    expect(changes).not.toHaveProperty("components");
    expect(changes.inputs).toEqual({});
    expect(document.sources()).toEqual({ remote: [], host: [] });
    expect(JSON.parse(document.reload({}).splice)).not.toHaveProperty("edit");
    document.free();
  });

  it("lists sources after an edit to the frontmatter, and reloads with what was fetched", () => {
    const document = new OpenDocument({ ...request("Text.\n"), allowedOrigins: ["https://a.org"] });
    const frontmatter = "---\nmd:\n  inputs:\n    n: number = 1\n  data:\n    - from: https://a.org/x.json\n      as: x\n---\n";
    document.edit(0, 0, frontmatter);
    document.edit(frontmatter.length + 6, frontmatter.length + 6, "\n{{md.data.x.n}}\n");
    expect(document.sources()).toEqual({ remote: ["https://a.org/x.json"], host: [] });
    expect(document.expanded.inputs?.["n"]).toMatchObject({ value: 1 });
    expect(document.diagnostics[0]?.code).toBe("F026");
    const changes = document.reload({ remote: { "https://a.org/x.json": { text: '{"n": "fetched"}', contentType: "application/json" } } });
    expect(changes.splice).toMatchObject({ from: 0, to: 2 });
    expect(document.expanded.children[1]).toMatchObject({ children: [{ value: "fetched" }] });
    expect(document.diagnostics).toEqual([]);
  });

  it("keeps the host-language components, which a change gives only when they changed", () => {
    const document = new OpenDocument(completeExample());
    expect(document.components.map((c) => [c.name, c.language])).toEqual([["board", "jsx"], ["chart", "jsx"], ["sparkline", "jsx"]]);
    const text = completeExample().files?.["input.comp.md"] ?? "";
    const at = text.indexOf("## What changed") + 3;
    expect(document.edit(at, at, "x").components).toBeUndefined();
    expect(document.components).toHaveLength(3);
  });

  it("gives the hint at a UTF-16 position of the text as the last edit left it", () => {
    const document = new OpenDocument(request("😀 :box[b]{border=subtle}\n"));
    // `subtle` starts at 18 in UTF-16 units: the emoji is two.
    const hint = document.hintAt(20);
    expect(hint?.subject).toMatchObject({ kind: "param", component: "box", value: "subtle", param: { name: "border" } });
    expect(hint?.range.start).toEqual({ line: 1, column: 18 });
    expect(hint?.completions.map((c) => c.text)).toEqual(["none", "subtle", "strong"]);
    document.edit(18, 24, "nope");
    expect(document.hintAt(20)?.diagnostics.map((d) => d.code)).toEqual(["F009"]);
    expect(new OpenDocumentJson(request("Plain.\n")).hintAt(2)).toBeUndefined();
    document.free();
  });

  it("throws on a request it can't open", () => {
    expect(() => new OpenDocument({ path: "a.comp.md", files: {}, now })).toThrow("`a.comp.md` isn't one of the request's files");
  });
});
