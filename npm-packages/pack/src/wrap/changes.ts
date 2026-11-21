// What a build of a wrapped pack says changed (docs/wrapping-an-npm-package.md, decisions 4 and 5): a param added, one
// gone, one whose declaration changed, from the last generated fence to this one; and the package's props the allowlist
// doesn't name, which are reported, never added.

import { parse } from "yaml";

/** A signature fence's params, each as its declaration's text: the short form, or the long form's type and default. */
export function declarationsOf(fence: string): Map<string, string> {
  const lines = fence.split("\n");
  let header: unknown;
  try {
    header = parse(lines.slice(1, -1).join("\n"));
  } catch {
    return new Map();
  }
  const params = typeof header === "object" && header !== null ? (header as { params?: unknown }).params : undefined;
  if (typeof params !== "object" || params === null) return new Map();
  return new Map(
    Object.entries(params).map(([name, declared]) => {
      if (typeof declared === "string") return [name, declared];
      const long = (declared ?? {}) as { type?: unknown; default?: unknown };
      return [name, `${String(long.type)}${long.default === undefined ? "" : ` = ${JSON.stringify(long.default)}`}`];
    }),
  );
}

/**
 * What changed from the fence `before` to the fence `after`, one line each, in the order of `after`, then what went.
 *
 * @example
 * changesOf(before, after); // ["`level` is new: L | M | H | Q?", "`size` is gone"]
 */
export function changesOf(before: string, after: string): string[] {
  const old = declarationsOf(before);
  const now = declarationsOf(after);
  const changes: string[] = [];
  for (const [name, declared] of now) {
    const was = old.get(name);
    if (was === undefined) changes.push(`\`${name}\` is new: ${declared}`);
    else if (was !== declared) changes.push(`\`${name}\` was ${was}, now ${declared}`);
  }
  for (const name of old.keys()) if (!now.has(name)) changes.push(`\`${name}\` is gone`);
  return changes;
}

/**
 * Where the text of `files` (the demo, the overrides) still names a param that went: `name=` in a use, or the param as
 * a key of the overrides.
 */
export function stillNamed(gone: readonly string[], files: Readonly<Record<string, string>>): string[] {
  return gone.flatMap((name) =>
    Object.entries(files)
      .filter(([, text]) => new RegExp(`(?:[\\s{]${name}=|"${name}"\\s*:)`).test(text))
      .map(([file]) => `${file} still names \`${name}\`, which the package no longer has`),
  );
}
