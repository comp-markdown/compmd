// A table keyed by a name a document or a frame writes (audit S13), looked up among its own keys only: a name such as
// `constructor`, `__proto__` or `toString` never finds what every object inherits.

/**
 * `table[name]` when `name` is one of `table`'s own keys; `undefined` otherwise, and for a name every object inherits.
 *
 * @example
 * lookup({ mermaid: draw }, "mermaid"); // draw
 * lookup({ mermaid: draw }, "constructor"); // undefined
 */
export function lookup<T>(table: Readonly<Record<string, T>> | undefined, name: string): T | undefined {
  return table !== undefined && Object.hasOwn(table, name) ? table[name] : undefined;
}
