import { initSync, process } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { EXAMPLES, exampleIndex, textOf, type Example } from "./documents.ts";
import { fixOf } from "./edit.ts";
import { findPassage, parseFragment } from "./fragment.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const byId = (id: string): Example => {
  const found = EXAMPLES.find((example) => example.id === id);
  if (!found) throw new Error(id);
  return found;
};

/** The diagnostics of `example` with its document's text `text`, one line each. */
const diagnosticsOf = (example: Example, text = textOf(example)) =>
  process({ ...example.request, files: { ...example.request.files, [example.request.path]: text } }).diagnostics;

describe("EXAMPLES", () => {
  it("go from the syntax to the diagnostics, each with an id of its own", () => {
    expect(EXAMPLES.map((example) => example.id)).toEqual(["tour", "definitions", "changelog", "dashboard", "inputs", "kyoto", "complete", "diagnostics", "notes"]);
    expect(exampleIndex("kyoto")).toBe(5);
    expect(exampleIndex("nothing")).toBe(0);
    expect(exampleIndex(null)).toBe(0);
  });

  for (const example of EXAMPLES.filter(({ id }) => id !== "diagnostics")) {
    it(`${example.name} renders without diagnostics`, () => {
      const processed = process(example.request);
      const found = processed.diagnostics.map((d) => `${d.code} ${String(d.range.start.line)}: ${d.message}`);
      expect(found, found.join("\n")).toEqual([]);
      expect(processed.expanded.children.length).toBeGreaterThan(1);
    });
  }

  it("shows mistakes whose fixes, made one by one, leave none", () => {
    const example = byId("diagnostics");
    let text = textOf(example);
    const first = diagnosticsOf(example);
    expect(first.map((d) => d.code).sort()).toEqual(["F002", "F006", "F008", "F009", "F009"]);
    for (let round = 0; round < first.length; round++) {
      const now = diagnosticsOf(example, text);
      const fix = now[0]?.fix;
      if (!fix) throw new Error(`no fix in round ${String(round)}: ${JSON.stringify(now)}`);
      const { start, end, text: inserted } = fixOf(text, fix);
      text = text.slice(0, start) + inserted + text.slice(end);
    }
    expect(diagnosticsOf(example, text)).toEqual([]);
  });

  it("links the tutorial's steps to passages of its text, each found where it isn't the link itself", () => {
    const text = textOf(byId("tour"));
    // Each link destination that is a text fragment, its parentheses balanced, as CommonMark reads a destination.
    const links: string[] = [];
    for (let at = text.indexOf("](#:~:text="); at !== -1; at = text.indexOf("](#:~:text=", at + 1)) {
      let depth = 0;
      let end = at + 2;
      for (; end < text.length; end++) {
        if (text[end] === "(") depth++;
        else if (text[end] === ")" && depth-- === 0) break;
      }
      links.push(text.slice(at + 2, end));
    }
    expect(links.length).toBeGreaterThan(15);
    for (const link of links) {
      const directive = parseFragment(link);
      const passage = directive && findPassage(text, directive);
      expect(passage, link).toBeDefined();
      if (!passage) continue;
      const line = text.slice(text.lastIndexOf("\n", passage.start) + 1, text.indexOf("\n", passage.start));
      expect(line, link).not.toContain(link);
    }
  });

  it("gives the dashboard and the field notes the charts with their code", () => {
    const charts = EXAMPLES.filter((example) => example.request.packs?.["@preview/charts"]);
    expect(charts.map((example) => example.id)).toEqual(["dashboard", "notes"]);
    const files = Object.keys(charts[0]?.request.packs?.["@preview/charts"]?.[0]?.files ?? {});
    expect(files.sort()).toEqual(["area-chart.jsx", "bar-chart.jsx", "data-table.svelte", "index.comp.md", "package.json", "pie-chart.svelte", "timeline.jsx"]);
  });

  it("list the files each document reads: its folder's, its packs', its remote and host data", () => {
    const names = (id: string) => byId(id).files.map((file) => file.name);
    expect(names("tour")).toEqual(["books.csv"]);
    expect(names("diagnostics")).toEqual([]);
    expect(names("dashboard")).toEqual(expect.arrayContaining(["sales.csv", "milestones.yaml", "@preview/charts@1.0.0/bar-chart.jsx"]));
    const complete = names("complete");
    expect(complete).toEqual(
      expect.arrayContaining(["board.jsx", "team.yaml", "@acme/charts@1.4.2/chart.jsx", "https://status.rayplay.dev/today.json?region=all", "host:tasks?project=rayplay"]),
    );
    const caseFiles = ["case.json", "context.json", "expanded.json", "status-today.json", "tasks.json"];
    expect(complete.filter((name) => caseFiles.includes(name) || name.startsWith("registry/"))).toEqual([]);
    expect(byId("complete").files.find((file) => file.name.startsWith("host:"))?.text).toContain('"title"');
  });
});
