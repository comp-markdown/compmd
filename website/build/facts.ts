// What the landing page says of the repository, counted when the site is built rather than written by hand, so it
// can't go stale: the conformance set's cases, and the processor's size as WebAssembly, gzipped.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { gzipSync } from "node:zlib";

/** What the page says of the repository. */
export interface Facts {
  /** How many cases the conformance set has: one `case.json` each. */
  cases: number;
  /** The processor's WebAssembly, gzipped, in KB (1,000 bytes), as a page weighs it. */
  wasmKb: number;
}

/** How many cases there are under `folder`: its `case.json` files, at any depth. */
export function casesIn(folder: string): number {
  return readdirSync(folder, { recursive: true, encoding: "utf8" }).filter((path) => path === "case.json" || path.endsWith("/case.json") || path.endsWith("\\case.json")).length;
}

/** `file`'s size gzipped, in KB of 1,000 bytes, rounded. */
export function gzippedKb(file: string): number {
  return Math.round(gzipSync(readFileSync(file), { level: 9 }).length / 1000);
}

/** The facts of the repository at `repo`, its root. */
export function factsOf(repo: string): Facts {
  return { cases: casesIn(join(repo, "conformance")), wasmKb: gzippedKb(join(repo, "wasm/pkg/compmd_wasm_bg.wasm")) };
}
