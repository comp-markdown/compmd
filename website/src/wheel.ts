// Where the reader is among the landing page's sections, as a number the contents wheel turns by: 2 at the top of the
// third section, 2.5 halfway through it.

/**
 * The reader's place among sections whose tops are `tops` (in the page's order, relative to the window) when the
 * reading line is at `line`: the index of the last section that has reached the line, plus how far through it the line
 * is. 0 before the first, and the last index at the end of the last.
 */
export function placeOf(tops: readonly number[], line: number, bottom: number): number {
  let index = 0;
  for (let i = 0; i < tops.length; i++) if ((tops[i] ?? 0) <= line) index = i;
  const top = tops[index] ?? 0;
  const next = tops[index + 1] ?? bottom;
  if (top > line || next <= top) return index;
  return Math.min(tops.length - 1, index + Math.min(1, (line - top) / (next - top)));
}

/** How an item `distance` sections from the reader's place looks: its offset in rows, its size and its opacity. */
export function lookOf(distance: number): { offset: number; scale: number; opacity: number } {
  const away = Math.min(Math.abs(distance), 4);
  return { offset: distance, scale: 1 - away * 0.08, opacity: Math.max(0, 1 - away * 0.22) };
}
