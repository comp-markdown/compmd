// An error in a ```wireframe block (docs/wireframe-language.md, decision 6): where it is, in the block's lines and
// columns from 1, and what to write instead.

/** What is wrong in a source, and where. */
export class WireframeError extends Error {
  constructor(
    readonly line: number,
    readonly column: number,
    readonly problem: string,
  ) {
    super(`line ${String(line)}, column ${String(column)}: ${problem}`);
    this.name = "WireframeError";
  }
}

/** The edit distance between `a` and `b`, for "did you mean": insertions, deletions and substitutions. */
export function distance(a: string, b: string): number {
  let previous = Array.from({ length: b.length + 1 }, (_, i) => i);
  for (let i = 1; i <= a.length; i++) {
    const current = [i];
    for (let j = 1; j <= b.length; j++) current[j] = Math.min((previous[j] ?? 0) + 1, (current[j - 1] ?? 0) + 1, (previous[j - 1] ?? 0) + (a[i - 1] === b[j - 1] ? 0 : 1));
    previous = current;
  }
  return previous[b.length] ?? 0;
}

/** The name of `names` nearest to `name`, if one is near enough to be a slip: `: did you mean \`button\`?`, or nothing. */
export function suggestion(name: string, names: Iterable<string>): string {
  let best: [string, number] | undefined;
  for (const candidate of names) {
    const d = distance(name, candidate);
    if (d <= Math.max(1, Math.floor(candidate.length / 3)) && (!best || d < best[1])) best = [candidate, d];
  }
  return best ? `: did you mean \`${best[0]}\`?` : "";
}
