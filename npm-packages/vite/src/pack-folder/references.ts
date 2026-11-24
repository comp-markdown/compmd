// What a pack's document names relative to itself, as spec §8.6 resolves it against the pack: the documents and
// component files it imports, its data files and their fallbacks, its fonts' files, and the link and image
// destinations in its `compmd` definitions.

import { parse as parseYaml } from "yaml";
import { joinedInPack } from "./confined.ts";

/** What a document names in its pack, each by its path in the pack. */
export interface References {
  /** Its relative imports: documents (`.comp.md`) and component files. */
  imports: string[];
  /** Its data files and their fallbacks. */
  data: string[];
  /** Its fonts' files. */
  fonts: string[];
  /** The files its definitions' links and images name. */
  destinations: string[];
  /** What it writes that leaves the pack, or isn't a path in it, as written. */
  outside: string[];
  /**
   * The paths no one can list before the pack is used, as written: a definition's destination or a data `from` that
   * interpolates after literal text (`./icons/{{icon}}.svg`).
   */
  unlisted: string[];
}

const FRONTMATTER = /^---\r?\n([\s\S]*?)\r?\n---\r?\n/;
/** A ` ```compmd ` (or `~~~`) block and its info string, which a definition's has `component=` in. */
const COMPMD_BLOCK = /^(`{3,}|~{3,})compmd\b([^\n]*)\n([\s\S]*?)^\1[ \t]*$/gm;
/** An inline link's or image's destination: `[text](./a.svg "title")`, `![](<./b c.png>)`. */
const INLINE_DESTINATION = /\]\(\s*(?:<([^>\n]*)>|([^)\s]+))/g;
/** A link reference definition's destination: `[star]: ./icons/star.svg`. */
const DEFINED_DESTINATION = /^[ ]{0,3}\[[^\]\n]+\]:[ \t]*(?:<([^>\n]*)>|(\S+))/gm;

/** The `md` key of a document's frontmatter, or an empty one; the processor reports a broken frontmatter. */
export function mdOf(text: string): Record<string, unknown> {
  try {
    const md = (parseYaml(FRONTMATTER.exec(text)?.[1] ?? "") as { md?: unknown } | null)?.md;
    return typeof md === "object" && md !== null && !Array.isArray(md) ? (md as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

/** Whether `from` is a relative path, `./x` or `../x`, as an import's or a data entry's is. */
const isRelative = (from: string): boolean => from.startsWith("./") || from.startsWith("../");

/** The entries of a list, `[]` for anything else. */
const listOf = (value: unknown): unknown[] => (Array.isArray(value) ? (value as unknown[]) : []);

/** `entry[key]` when it is a string. */
function stringAt(entry: unknown, key: string): string | undefined {
  const value = typeof entry === "object" && entry !== null ? (entry as Record<string, unknown>)[key] : undefined;
  return typeof value === "string" ? value : undefined;
}

/** Each written path the frontmatter names relative to the document, by what it is. */
function frontmatterPaths(md: Record<string, unknown>): { imports: string[]; data: string[]; fonts: string[] } {
  const imports = listOf(md.import).flatMap((entry) => stringAt(entry, "from") ?? []).filter(isRelative);
  const data = listOf(md.data).flatMap((entry) => [stringAt(entry, "from"), stringAt(entry, "fallback")].filter((from): from is string => from !== undefined && isRelative(from)));
  const fonts = Object.values(typeof md.fonts === "object" && md.fonts !== null ? md.fonts : {}).flatMap((font: unknown) => listOf((font as { files?: unknown } | null)?.files).flatMap((file) => stringAt(file, "from") ?? []));
  return { imports, data, fonts };
}

/** The relative link and image destinations the `compmd` definitions in `text` write, without query or fragment. */
function definitionDestinations(text: string): string[] {
  const destinations: string[] = [];
  for (const [, , info = "", body = ""] of text.matchAll(COMPMD_BLOCK)) {
    if (!/\bcomponent=/.test(info)) continue;
    for (const pattern of [INLINE_DESTINATION, DEFINED_DESTINATION]) {
      for (const [, angled, bare] of body.matchAll(pattern)) {
        const path = (angled ?? bare ?? "").split(/[?#]/)[0] ?? "";
        // A destination with a scheme, from a root, or a fragment alone isn't the pack's; one that starts with an
        // interpolation is the caller's (spec §8.6).
        if (path !== "" && !/^[a-z][a-z0-9+.-]*:|^\/|^\{\{/i.test(path)) destinations.push(path);
      }
    }
  }
  return destinations;
}

/**
 * What the document at `document` (a path in its pack), with `text`, names in its pack.
 *
 * @example
 * referencesOf("index.comp.md", text).imports; // ["parts/cards.comp.md", "dist/chart.js"]
 */
export function referencesOf(document: string, text: string): References {
  const references: References = { imports: [], data: [], fonts: [], destinations: [], outside: [], unlisted: [] };
  const written = { ...frontmatterPaths(mdOf(text)), destinations: definitionDestinations(text) };
  for (const kind of ["imports", "data", "fonts", "destinations"] as const) {
    for (const path of written[kind]) {
      // A destination ending in `/` names a folder, which publishes nothing of its own.
      if (kind === "destinations" && path.endsWith("/")) continue;
      if (path.includes("{{")) {
        references.unlisted.push(path);
        continue;
      }
      const joined = joinedInPack(document, path);
      if (joined === undefined) references.outside.push(path);
      else references[kind].push(joined);
    }
  }
  return references;
}
