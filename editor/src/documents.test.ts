import { initSync, process, type PackRelease, type Request } from "@compmd/processor";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { findPassage, fixOf, parseFragment } from "@compmd/devtools/testing";
import { EXAMPLES, exampleIndex, textOf, type Example } from "./documents.ts";

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const byId = (id: string): Example => {
  const found = EXAMPLES.find((example) => example.id === id);
  if (!found) throw new Error(id);
  return found;
};

/** The Mermaid pack as npm gives it, from this repository's `packs/mermaid/`: its built module given by its place. */
const MERMAID = join(import.meta.dirname, "../../packs/mermaid");
const mermaidRelease = (): PackRelease => {
  const manifest = readFileSync(join(MERMAID, "package.json"), "utf8");
  return {
    version: (JSON.parse(manifest) as { version: string }).version,
    integrity: "sha512-from-the-repository",
    files: { "package.json": manifest, "index.comp.md": readFileSync(join(MERMAID, "index.comp.md"), "utf8"), "dist/mermaid.js": "" },
  };
};

/** `request` with the packs the playground would fetch from npm for it, here from the repository. */
const withRegistryPacks = (request: Request): Request => ({ ...request, packs: { ...request.packs, "@comp-markdown/mermaid": [mermaidRelease()] } });

/** npm's registry, which the document of data from the web reads once the reader allows it. */
const NPM = "https://registry.npmjs.org";
const LATEST = `${NPM}/@comp-markdown/mermaid/latest`;

/** `request` as the reader's Allow leaves it: npm's registry allowed, and its answer fetched. */
const withNpmAllowed = (request: Request): Request => ({
  ...request,
  allowedOrigins: [...(request.allowedOrigins ?? []), NPM],
  remote: { ...request.remote, [LATEST]: { text: JSON.stringify({ name: "@comp-markdown/mermaid", version: "9.9.9", description: "From npm." }), contentType: "application/json" } },
});

/** The diagnostics of `example` with its document's text `text`, one line each. */
const diagnosticsOf = (example: Example, text = textOf(example)) =>
  process({ ...example.request, files: { ...example.request.files, [example.request.path]: text } }).diagnostics;

describe("EXAMPLES", () => {
  it("go from the syntax to the diagnostics, each with an id of its own", () => {
    expect(EXAMPLES.map((example) => example.id)).toEqual([
      ...["tour", "book-club", "definitions", "changelog", "dashboard", "inputs", "kyoto", "complete", "diagrams", "npm", "remote", "diagnostics", "notes"],
      ...["a-pack", "breakpoints", "code-blocks", "containers", "data", "definitions", "first-document", "in-code", "inputs", "layout", "lists"].map((name) => `guides/authors/${name}`),
      ...["markdown", "placing", "showing", "signatures", "slots", "template-slots", "values"].map((name) => `guides/authors/${name}`),
      ...["check-docs", "devtools", "live-editor", "orders", "other-hosts", "packs-from-npm", "quick-start", "styled-page", "two-modes"].map((name) => `guides/developers/${name}`),
    ]);
    expect(exampleIndex("guides/developers/quick-start")).toBe(37);
    expect(exampleIndex("kyoto")).toBe(6);
    expect(exampleIndex("nothing")).toBe(0);
    expect(exampleIndex(null)).toBe(0);
  });

  for (const example of EXAMPLES.filter(({ id }) => id !== "diagnostics")) {
    it(`${example.name} renders without diagnostics`, () => {
      const processed = process(withNpmAllowed(withRegistryPacks(example.request)));
      const found = processed.diagnostics.map((d) => `${d.code} ${String(d.range.start.line)}: ${d.message}`);
      expect(found, found.join("\n")).toEqual([]);
      expect(processed.expanded.children.length).toBeGreaterThan(1);
    });
  }

  it("gives the document of a pack from npm only the configured packs: until it is fetched, its import is F026", () => {
    const { diagnostics, expanded } = process(byId("npm").request);
    expect(diagnostics.map(({ code, message }) => `${code} ${message}`)).toEqual([expect.stringMatching(/^F026 .*@comp-markdown\/mermaid/)]);
    expect(expanded.children.length).toBeGreaterThan(1);
  });

  it("gives the document of data from the web its fallback until the reader allows npm's registry, then npm's answer", () => {
    const before = process(byId("remote").request);
    expect(before.diagnostics.map(({ code, message }) => `${code} ${message}`)).toEqual([expect.stringMatching(/^F028 .*registry\.npmjs\.org/)]);
    expect(JSON.stringify(before.expanded)).toContain("0.1.1 (a saved copy)");
    const after = process(withNpmAllowed(byId("remote").request));
    expect(JSON.stringify(after.expanded)).toContain("9.9.9");
  });

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

  it("gives every document the playground's packs from its configuration, the charts and the website's stars ambient", () => {
    for (const example of EXAMPLES) {
      expect(example.request.ambient, example.id).toEqual(["@preview/charts", "@site/app"]);
      expect(Object.keys(example.request.packs ?? {}), example.id).toEqual(expect.arrayContaining(["@preview/charts", "@preview/minimermaid", "@site/app"]));
      // The website trusts its own pack; in the playground it is sandboxed, as every pack there is.
      expect(example.request.policy?.packs, example.id).toEqual({ "@preview/charts": { level: "strict" }, "@preview/minimermaid": { level: "strict" }, "@site/app": { level: "strict" } });
    }
    const files = Object.keys(byId("dashboard").request.packs?.["@preview/charts"]?.[0]?.files ?? {});
    expect(files.sort()).toEqual(["area-chart.jsx", "bar-chart.jsx", "data-table.svelte", "index.comp.md", "package.json", "pie-chart.svelte", "timeline.jsx"]);
    // The complete example's own packs come with it, beside the configured ones.
    expect(Object.keys(byId("complete").request.packs ?? {})).toEqual(expect.arrayContaining(["@acme/charts", "@preview/charts"]));
  });

  it("gives the diagrams the built module by its place, not its text", () => {
    const [release] = byId("diagrams").request.packs?.["@preview/minimermaid"] ?? [];
    expect(release?.files["dist/minimermaid.js"]).toBe("");
    expect(release?.files["index.comp.md"]).toContain("renders: mermaid");
  });

  it("list the files each document reads: its folder's, its packs', its remote and host data", () => {
    const names = (id: string) => byId(id).files.map((file) => file.name);
    expect(names("tour")).toEqual(["books.csv"]);
    expect(names("diagnostics")).toEqual([]);
    expect(names("dashboard")).toEqual(expect.arrayContaining(["sales.csv", "milestones.yaml", "@preview/charts@1.0.0/bar-chart.jsx"]));
    // Only the configured packs a document shows; a built module, given as no text, isn't listed.
    expect(names("diagrams")).toEqual(["@preview/minimermaid@0.1.0/package.json", "@preview/minimermaid@0.1.0/index.comp.md"]);
    const complete = names("complete");
    expect(complete).toEqual(
      expect.arrayContaining(["board.jsx", "team.yaml", "@acme/charts@1.4.2/chart.jsx", "https://status.rayplay.dev/today.json?region=all", "host:tasks?project=rayplay"]),
    );
    const caseFiles = ["case.json", "context.json", "expanded.json", "status-today.json", "tasks.json"];
    expect(complete.filter((name) => caseFiles.includes(name) || name.startsWith("registry/"))).toEqual([]);
    expect(byId("complete").files.find((file) => file.name.startsWith("host:"))?.text).toContain('"title"');
  });
});
