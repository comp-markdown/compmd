// A table keyed by a name a wireframe writes (audit S13), looked up among its own keys only: a part or an attribute
// named `constructor`, `__proto__` or `toString` never finds what every object inherits.

/** `table[name]` when `name` is one of `table`'s own keys; `undefined` otherwise. */
export function lookup<T>(table: Readonly<Record<string, T>>, name: string): T | undefined {
  return Object.hasOwn(table, name) ? table[name] : undefined;
}
