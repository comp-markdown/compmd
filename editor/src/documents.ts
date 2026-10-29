// The documents the editor opens, each showing one side of CompMD, bundled with everything they need: their files,
// packs, remote and host data. The editor fetches nothing.

import type { Fetched, PackRelease, Request } from "@compmd/processor";

export interface Example {
  /** What `?doc=` names it by. */
  id: string;
  name: string;
  request: Request;
  /** What the document reads besides itself, shown read-only beside it: its files, its packs' files, its remote and host data. */
  files: SupportingFile[];
}

/** A file a document reads, by the name it is shown under. */
export interface SupportingFile {
  name: string;
  text: string;
}

/** Files bundled as text, by path from this file. */
type Bundle = Record<string, string>;

const COMPLETE = "../../conformance/examples/complete-example-expanded/";
const CHANGELOG = "../../wasm/bench/fixtures/changelog/";
const NOTES = "../../wasm/bench/fixtures/notes/";
const EXAMPLES_DIR = "../examples/";
const CHARTS = `${EXAMPLES_DIR}charts/`;

const bundled = {
  complete: import.meta.glob("../../conformance/examples/complete-example-expanded/**/*", { query: "?raw", import: "default", eager: true }),
  changelog: import.meta.glob("../../wasm/bench/fixtures/changelog/*", { query: "?raw", import: "default", eager: true }),
  // The notes alone: their pack holds signatures only, and the examples' charts implement them.
  notes: import.meta.glob("../../wasm/bench/fixtures/notes/notes.comp.md", { query: "?raw", import: "default", eager: true }),
  examples: import.meta.glob("../examples/**/*", { query: "?raw", import: "default", eager: true }),
} as Record<string, Bundle>;

/** The files of `bundle` under `dir`, by their path from it. */
function filesIn(bundle: Bundle | undefined, dir: string): Record<string, string> {
  return Object.fromEntries(
    Object.entries(bundle ?? {})
      .filter(([path]) => path.startsWith(dir))
      .map(([path, text]) => [path.slice(dir.length), text]),
  );
}

/** The files of the editor's own example in `examples/<dir>/`. */
const own = (dir: string) => filesIn(bundled.examples, `${EXAMPLES_DIR}${dir}/`);

interface CaseContext {
  now: string;
  allowedOrigins: string[];
  registry: Record<string, { version: string; path: string; integrity: string }[]>;
  urls: Record<string, string>;
  host: Record<string, string>;
}

/** The files of the complete example's folder that are its conformance case's, not the document's. */
const CASE_FILES = new Set(["case.json", "context.json", "expanded.json"]);

/** The spec's complete example, with the packs, remote and host data its conformance case gives. */
function completeExample(): Request {
  const folder = filesIn(bundled.complete, COMPLETE);
  const context = JSON.parse(folder["context.json"] ?? "{}") as CaseContext;
  const packs: Record<string, PackRelease[]> = {};
  for (const [pack, releases] of Object.entries(context.registry)) {
    packs[pack] = releases.map(({ version, path, integrity }) => ({ version, integrity, files: filesIn(bundled.complete, `${COMPLETE}${path}/`) }));
  }
  // The case's remote and host data are files of its folder, which the document reads as what they stand for.
  const standIns = new Set([...Object.values(context.urls), ...Object.values(context.host)]);
  const files = Object.fromEntries(Object.entries(folder).filter(([path]) => !CASE_FILES.has(path) && !standIns.has(path) && !path.startsWith("registry/")));
  const fetched = (file: string): Fetched => ({ text: folder[file] ?? "", contentType: "application/json" });
  return {
    path: "input.comp.md",
    files,
    packs,
    allowedOrigins: context.allowedOrigins,
    remote: Object.fromEntries(Object.entries(context.urls).map(([url, file]) => [url, fetched(file)])),
    host: Object.fromEntries(Object.entries(context.host).map(([source, file]) => [source, JSON.parse(folder[file] ?? "null") as unknown])),
    now: context.now,
  };
}

/** The examples' charts, an ambient pack with their code, under the name the field notes give theirs. */
const CHARTS_PACK = "@preview/charts";

/** The document `path` of `files`, with the examples' charts. */
function withCharts(files: Record<string, string>, path: string, now: string): Request {
  return {
    path,
    files,
    packs: { [CHARTS_PACK]: [{ version: "1.0.0", files: filesIn(bundled.examples, CHARTS) }] },
    ambient: [CHARTS_PACK],
    now,
  };
}

/** What `request`'s document reads besides itself: its folder's other files, its packs' files, its remote and host data. */
export function supportingFiles(request: Request): SupportingFile[] {
  const files = Object.entries(request.files ?? {})
    .filter(([path]) => path !== request.path)
    .map(([name, text]) => ({ name, text }));
  const packs = Object.entries(request.packs ?? {}).flatMap(([pack, releases]) =>
    releases.flatMap(({ version, files: packFiles }) => Object.entries(packFiles).map(([path, text]) => ({ name: `${pack}@${version}/${path}`, text }))),
  );
  const remote = Object.entries(request.remote ?? {}).map(([url, { text }]) => ({ name: url, text }));
  const host = Object.entries(request.host ?? {}).map(([source, value]) => ({ name: `host:${source}`, text: JSON.stringify(value, null, 2) }));
  return [...files, ...packs, ...remote, ...host];
}

/** An example of `request`, with its supporting files. */
const example = (id: string, name: string, request: Request): Example => ({ id, name, request, files: supportingFiles(request) });

const NOW = "2026-10-29T09:00:00Z";

/** The menu, from the syntax to the diagnostics, each document showing one side of CompMD. */
export const EXAMPLES: Example[] = [
  example("tour", "Learn CompMD: a tutorial in fourteen lessons", { path: "tour.comp.md", files: own("tour"), now: NOW }),
  example("definitions", "Definitions and slots", { path: "definitions.comp.md", files: own("definitions"), now: NOW }),
  example("changelog", "Expressions: a changelog (12 releases, 128 changes)", { path: "changelog.comp.md", files: filesIn(bundled.changelog, CHANGELOG), now: "2026-10-22T09:00:00Z" }),
  example("dashboard", "Data: a sales dashboard", withCharts(own("dashboard"), "dashboard.comp.md", NOW)),
  example("inputs", "Inputs: plan a stay", { path: "planner.comp.md", files: own("inputs"), now: NOW }),
  example("kyoto", "Layout: Kyoto in four days", { path: "kyoto.comp.md", files: own("kyoto"), now: NOW }),
  example("complete", "Components in JSX and from packs: the spec's complete example", completeExample()),
  example("diagnostics", "Diagnostics and their fixes", { path: "diagnostics.comp.md", files: own("diagnostics"), now: NOW }),
  example("notes", "A long document: field notes (52 KB, 20 charts)", withCharts(filesIn(bundled.notes, NOTES), "notes.comp.md", "2026-10-23T09:00:00Z")),
];

/** The example `?doc=` names, or the first. */
export function exampleIndex(id: string | null): number {
  return Math.max(0, EXAMPLES.findIndex((example) => example.id === id));
}

/** The text of an example's document. */
export const textOf = ({ request }: Example): string => request.files?.[request.path] ?? "";
