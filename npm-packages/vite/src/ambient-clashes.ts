// A name two ambient packs export (spec §6.3): a document that uses it gets F023 and the component's fallback, whatever
// order the configuration lists the packs in. The build says so when it loads the app, so a host sees the clash before
// any document does. Each ambient pack's names are what the processor lists for a document with that pack alone
// ambient, each by the file that defines it, so a pack that re-exports another's component is no clash.

import { componentsOf, type PackRelease, type Policy } from "@compmd/processor";

/** What the processor is given to list a pack's names: the app's packs and policy. */
export interface ClashRequest {
  packs: Record<string, PackRelease[]>;
  policy: Policy;
  ambient: string[];
}

/** `@compmd/core`, which the processor provides to every document and which is no ambient pack. */
const CORE = "@compmd/core@";

/**
 * A warning for each name two of `request`'s ambient packs export, naming the packs; none when they share no name.
 *
 * @example
 * ambientClashes({ packs, policy, ambient: ["@acme/fields", "@kit/fields"] });
 * // ["`field` is exported by two ambient packs, @acme/fields and @kit/fields: …"]
 */
export function ambientClashes(request: ClashRequest): string[] {
  if (request.ambient.length < 2) return [];
  /** Each name's defining files, and the ambient packs that give it. */
  const byName = new Map<string, { files: Set<string>; packs: string[] }>();
  for (const pack of request.ambient) {
    const { groups } = componentsOf({ path: "clash.comp.md", files: { "clash.comp.md": "" }, packs: request.packs, policy: request.policy, ambient: [pack], now: new Date(0).toISOString() });
    for (const { file, provided, components } of groups) {
      if (!provided || file === undefined || file.startsWith(CORE)) continue;
      for (const { name } of components) {
        const found = byName.get(name) ?? { files: new Set<string>(), packs: [] };
        found.files.add(file);
        if (!found.packs.includes(pack)) found.packs.push(pack);
        byName.set(name, found);
      }
    }
  }
  return [...byName]
    .filter(([, { files }]) => files.size > 1)
    .map(([name, { packs }]) => {
      const sorted = [...packs].sort();
      const listed = sorted.length === 2 ? sorted.join(" and ") : `${sorted.slice(0, -1).join(", ")} and ${sorted.at(-1) ?? ""}`;
      return `\`${name}\` is exported by ${String(sorted.length)} ambient packs, ${listed}: a document that uses it gets F023 and its fallback, until it imports the name from one of them`;
    });
}
