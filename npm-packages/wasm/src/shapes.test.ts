// What the binding gives against what `index.ts` says it gives (audit A12): the types are written by hand, so a field
// the Rust side adds, renames or drops would go unnoticed. Each type's keys are listed once, checked by the compiler
// against the type; each value the binding gives must have only those keys, and every one the type requires.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { initSync, process, sources, type CacheKey, type Diagnostic, type Needed, type NeededPack, type Pending, type Processed, type Request } from "./index.ts";

/** The keys of `T` that aren't optional. */
type RequiredKeys<T> = { [K in keyof T]-?: object extends Pick<T, K> ? never : K }[keyof T];

/** A type's keys, all of them and those it requires: the compiler refuses a list that misses one or adds one. */
interface Shape<T> {
  all: Record<keyof T, true>;
  required: Record<RequiredKeys<T>, true>;
}

/** `value`, an object the binding gave, has only `shape`'s keys, and all those it requires. */
function expectShape<T>(value: unknown, { all, required }: Shape<T>, label: string): void {
  const keys = Object.keys(value as object);
  expect(keys.filter((key) => !(key in all)), `${label}: keys index.ts doesn't have`).toEqual([]);
  expect(Object.keys(required).filter((key) => !keys.includes(key)), `${label}: keys index.ts requires`).toEqual([]);
}

const PROCESSED: Shape<Processed> = {
  all: { parse: true, expanded: true, diagnostics: true, failed: true, cacheKeys: true, pending: true },
  required: { parse: true, expanded: true, diagnostics: true, failed: true, cacheKeys: true, pending: true },
};
const DIAGNOSTIC: Shape<Diagnostic> = {
  all: { code: true, severity: true, message: true, file: true, range: true, related: true, stage: true, fix: true },
  required: { code: true, severity: true, message: true, range: true, stage: true },
};
const CACHE_KEY: Shape<CacheKey> = { all: { as: true, key: true }, required: { as: true, key: true } };
const PENDING: Shape<Pending> = { all: { packs: true, remote: true, host: true }, required: { packs: true, remote: true, host: true } };
const NEEDED: Shape<Needed> = { all: { remote: true, host: true, disallowed: true, packs: true }, required: { remote: true, host: true, disallowed: true, packs: true } };
const NEEDED_PACK: Shape<NeededPack> = { all: { pack: true, range: true, integrity: true, library: true }, required: { pack: true, range: true } };

/** A document with a data import, a pending pack, a pinned import of a pack the request hasn't, and a misspelled component. */
const TEXT = [
  "---",
  "md:",
  "  import:",
  '    - from: "@kit/charts@1"',
  "      names: [chart]",
  '    - from: "@kit/theme@1.0.0"',
  "      integrity: sha512-t",
  '      names: "*"',
  "  data:",
  "    - from: ./notes.json",
  "      as: notes",
  "---",
  "",
  ":: chart[Sales by month]",
  "",
  ":: bx[A box]",
  "",
].join("\n");

const REQUEST: Request = { path: "a.comp.md", files: { "a.comp.md": TEXT, "notes.json": "[]" }, now: "2026-10-22T09:00:00Z", pending: { packs: ["@kit/charts"] } };

beforeAll(() => {
  initSync({ module: readFileSync(join(import.meta.dirname, "../pkg/compmd_wasm_bg.wasm")) });
});

describe("what the binding gives", () => {
  it("has the shape index.ts gives process's result, its diagnostics, cache keys and pending", () => {
    const processed = process(REQUEST);
    expectShape(processed, PROCESSED, "Processed");
    expect(processed.diagnostics.length).toBeGreaterThan(0);
    for (const diagnostic of processed.diagnostics) expectShape(diagnostic, DIAGNOSTIC, `Diagnostic ${diagnostic.code}`);
    expect(processed.cacheKeys.length).toBeGreaterThan(0);
    for (const key of processed.cacheKeys) expectShape(key, CACHE_KEY, "CacheKey");
    expectShape(processed.pending, PENDING, "Pending");
  });

  it("has the shape index.ts gives sources's result and its packs", () => {
    const needed = sources(REQUEST);
    expectShape(needed, NEEDED, "Needed");
    expect(needed.packs.length).toBeGreaterThan(0);
    for (const pack of needed.packs) expectShape(pack, NEEDED_PACK, `NeededPack ${pack.pack}`);
  });
});
