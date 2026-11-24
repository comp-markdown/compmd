// Today as the writer's calendar has it, for the first post's date: not UTC's, which is another day near midnight.
// `@compmd/blog` has the same function; this package depends on nothing, so it keeps its own.

/**
 * `date`'s day in this machine's time zone, `YYYY-MM-DD`.
 *
 * @example
 * localDay(new Date(2026, 10, 23, 23, 30)); // "2026-11-23", in any time zone
 */
export function localDay(date: Date = new Date()): string {
  const two = (n: number): string => String(n).padStart(2, "0");
  return `${String(date.getFullYear())}-${two(date.getMonth() + 1)}-${two(date.getDate())}`;
}
