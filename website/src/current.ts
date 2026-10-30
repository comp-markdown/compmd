/**
 * The id of the section being read: the last of `ids`, in the page's order, whose heading's `top` is at or above
 * `line`; the first while none is. `undefined` for none.
 */
export function currentOf(ids: readonly string[], top: (id: string) => number, line: number): string | undefined {
  let current = ids[0];
  for (const id of ids) {
    if (top(id) > line) break;
    current = id;
  }
  return current;
}
