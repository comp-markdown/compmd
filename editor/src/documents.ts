// The documents the editor opens, bundled with everything they need: their files, packs, remote and host data. The
// editor fetches nothing.

import type { Fetched, PackRelease, Request } from "@compmd/processor";

export interface Example {
  name: string;
  request: Request;
}

/** Files bundled as text, by path from this file. */
type Bundle = Record<string, string>;

const COMPLETE = "../../conformance/examples/complete-example-expanded/";
const RELEASE_NOTES = "../../conformance/examples/spec-16-release-notes/";
const CHANGELOG = "../../wasm/bench/fixtures/changelog/";
const NOTES = "../../wasm/bench/fixtures/notes/";
const CHARTS = "../examples/charts/";
const DASHBOARD = "../examples/dashboard/";
const KYOTO = "../examples/kyoto/";

const bundled = {
  complete: import.meta.glob("../../conformance/examples/complete-example-expanded/**/*", { query: "?raw", import: "default", eager: true }),
  releaseNotes: import.meta.glob("../../conformance/examples/spec-16-release-notes/input.comp.md", { query: "?raw", import: "default", eager: true }),
  changelog: import.meta.glob("../../wasm/bench/fixtures/changelog/*", { query: "?raw", import: "default", eager: true }),
  // The notes alone: their pack holds signatures only, and the examples' charts implement them.
  notes: import.meta.glob("../../wasm/bench/fixtures/notes/notes.comp.md", { query: "?raw", import: "default", eager: true }),
  charts: import.meta.glob("../examples/charts/*", { query: "?raw", import: "default", eager: true }),
  dashboard: import.meta.glob("../examples/dashboard/**/*", { query: "?raw", import: "default", eager: true }),
  kyoto: import.meta.glob("../examples/kyoto/**/*", { query: "?raw", import: "default", eager: true }),
} as Record<string, Bundle>;

/** The files of `bundle` under `dir`, by their path from it. */
function filesIn(bundle: Bundle | undefined, dir: string): Record<string, string> {
  return Object.fromEntries(
    Object.entries(bundle ?? {})
      .filter(([path]) => path.startsWith(dir))
      .map(([path, text]) => [path.slice(dir.length), text]),
  );
}

interface CaseContext {
  now: string;
  allowedOrigins: string[];
  registry: Record<string, { version: string; path: string; integrity: string }[]>;
  urls: Record<string, string>;
  host: Record<string, string>;
}

/** The spec's complete example, with the packs, remote and host data its conformance case gives. */
function completeExample(): Request {
  const files = filesIn(bundled.complete, COMPLETE);
  const context = JSON.parse(files["context.json"] ?? "{}") as CaseContext;
  const packs: Record<string, PackRelease[]> = {};
  for (const [pack, releases] of Object.entries(context.registry)) {
    packs[pack] = releases.map(({ version, path, integrity }) => ({ version, integrity, files: filesIn(bundled.complete, `${COMPLETE}${path}/`) }));
  }
  const fetched = (file: string): Fetched => ({ text: files[file] ?? "", contentType: "application/json" });
  return {
    path: "input.comp.md",
    files,
    packs,
    allowedOrigins: context.allowedOrigins,
    remote: Object.fromEntries(Object.entries(context.urls).map(([url, file]) => [url, fetched(file)])),
    host: Object.fromEntries(Object.entries(context.host).map(([source, file]) => [source, JSON.parse(files[file] ?? "null") as unknown])),
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
    packs: { [CHARTS_PACK]: [{ version: "1.0.0", files: filesIn(bundled.charts, CHARTS) }] },
    ambient: [CHARTS_PACK],
    now,
  };
}

/** The menu, the complete example first. */
export const EXAMPLES: Example[] = [
  { name: "The spec's complete example", request: completeExample() },
  { name: "A sales dashboard", request: withCharts(filesIn(bundled.dashboard, DASHBOARD), "dashboard.comp.md", "2026-10-29T09:00:00Z") },
  { name: "A layout: Kyoto in four days", request: { path: "kyoto.comp.md", files: filesIn(bundled.kyoto, KYOTO), now: "2026-10-29T09:00:00Z" } },
  { name: "A changelog (12 releases, 128 changes)", request: { path: "changelog.comp.md", files: filesIn(bundled.changelog, CHANGELOG), now: "2026-10-22T09:00:00Z" } },
  { name: "Release notes (spec §16)", request: { path: "input.comp.md", files: filesIn(bundled.releaseNotes, RELEASE_NOTES), now: "2026-10-22T09:00:00Z" } },
  { name: "Field notes (52 KB, 20 charts)", request: withCharts(filesIn(bundled.notes, NOTES), "notes.comp.md", "2026-10-23T09:00:00Z") },
];

/** The text of an example's document. */
export const textOf = ({ request }: Example): string => request.files?.[request.path] ?? "";
