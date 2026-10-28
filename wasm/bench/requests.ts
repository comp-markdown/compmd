// The documents the bench times, as requests: the spec's complete example (from its conformance case), a changelog
// (12 releases, 128 changes) and a long body (52 KB).

import { readFileSync, readdirSync } from "node:fs";
import { join, relative, sep } from "node:path";
import type { PackRelease, Request } from "../src/index.ts";

const COMPLETE = join(import.meta.dirname, "../../conformance/examples/complete-example-expanded");
const FIXTURES = join(import.meta.dirname, "fixtures");

/** Every file under `dir`, by its `/`-separated path from `dir`. */
export function filesOf(dir: string): Record<string, string> {
  const files = readdirSync(dir, { recursive: true, withFileTypes: true }).filter((entry) => entry.isFile());
  return Object.fromEntries(
    files.map((entry) => {
      const path = join(entry.parentPath, entry.name);
      return [relative(dir, path).split(sep).join("/"), readFileSync(path, "utf8")];
    }),
  );
}

/** A JSON file of the complete example's case. */
export function completeJson(path: string): unknown {
  return JSON.parse(readFileSync(join(COMPLETE, path), "utf8"));
}

/** The spec's complete example, with the packs, remote and host data its conformance case gives. */
export function completeExample(): Request {
  const context = completeJson("context.json") as {
    now: string;
    allowedOrigins: string[];
    registry: Record<string, { version: string; path: string; integrity: string }[]>;
    urls: Record<string, string>;
    host: Record<string, string>;
  };
  const packs: Record<string, PackRelease[]> = {};
  for (const [pack, releases] of Object.entries(context.registry)) {
    packs[pack] = releases.map(({ version, path, integrity }) => ({ version, integrity, files: filesOf(join(COMPLETE, path)) }));
  }
  const fetched = (file: string) => ({ text: readFileSync(join(COMPLETE, file), "utf8"), contentType: "application/json" });
  return {
    path: "input.comp.md",
    files: filesOf(COMPLETE),
    packs,
    allowedOrigins: context.allowedOrigins,
    remote: Object.fromEntries(Object.entries(context.urls).map(([url, file]) => [url, fetched(file)])),
    host: Object.fromEntries(Object.entries(context.host).map(([source, file]) => [source, completeJson(file)])),
    now: context.now,
  };
}

/** A changelog of 12 releases and 128 changes, from two CSV files. */
export function changelog(): Request {
  return { path: "changelog.comp.md", files: filesOf(join(FIXTURES, "changelog")), now: "2026-10-22T09:00:00Z" };
}

/** A 52 KB body of prose, lists and charts from an ambient pack. */
export function longBody(): Request {
  const pack = filesOf(join(FIXTURES, "notes/preview-pack"));
  return {
    path: "notes.comp.md",
    files: { "notes.comp.md": readFileSync(join(FIXTURES, "notes/notes.comp.md"), "utf8") },
    packs: { "@preview/charts": [{ version: "1.0.0", files: pack }] },
    ambient: ["@preview/charts"],
    now: "2026-10-23T09:00:00Z",
  };
}
