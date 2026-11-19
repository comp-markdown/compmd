// The page's runs: short pieces of the repository's own TypeScript, as a diff viewer shows a review's hunks, the same
// every time, so each variant highlights the same text.

const sources = import.meta.glob(["../../../npm-packages/*/src/**/*.ts", "!**/*.test.ts"], { query: "?raw", import: "default", eager: true }) as Record<string, string>;

/** How many lines each run has, in turn: hunks of a few lines to a few dozen. */
const LENGTHS = [6, 12, 20, 9, 30, 15, 4, 25];

/** `count` runs of TypeScript, each a few lines to a few dozen. */
export function runsOf(count: number): string[] {
  const runs: string[] = [];
  let turn = 0;
  for (const path of Object.keys(sources).sort()) {
    const lines = (sources[path] ?? "").split("\n");
    for (let at = 0; at < lines.length && runs.length < count; ) {
      const length = LENGTHS[turn++ % LENGTHS.length] ?? 10;
      const run = lines.slice(at, at + length).join("\n").trimEnd();
      at += length;
      if (run.trim() !== "") runs.push(run);
    }
    if (runs.length === count) break;
  }
  return runs;
}
