// The publish check, the end of `compmd-pack build` (docs/writing-a-pack.md, decision 4): what a host would refuse,
// or render wrong, found before the pack is published.

import { existsSync, statSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { DEFAULT_MAX_PACK_SIZE } from "@compmd/registry";
import { isSkipped, mdOf, publishedFiles, reachOf, type Reach } from "@compmd/vite/pack-folder";
import { languageProblems, renderedLanguages } from "./languages.ts";
import type { Pack } from "./pack.ts";

/** Every file under `dir`, by its path from it, as npm publishes a package without `files`: not its dependencies or hidden files. */
async function everyFile(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  const pathOf = (entry: { parentPath: string; name: string }) => posix.relative(dir.split("\\").join("/"), join(entry.parentPath, entry.name).split("\\").join("/"));
  return entries.filter((entry) => entry.isFile()).map(pathOf).filter((path) => !path.split("/").some(isSkipped));
}

/** What npm publishes of the pack, by path in it: what `files` matches, with globs, or its whole folder without `files`. */
const publishedOf = async ({ dir, manifest }: Pack): Promise<string[]> => (await publishedFiles(dir, manifest.files)) ?? (await everyFile(dir));

/** About what npm unpacks of the pack, which a registry's `unpackedSize` says: what it publishes. */
export async function publishedSize(pack: Pack): Promise<number> {
  return (await publishedOf(pack)).reduce((total, path) => total + statSync(join(pack.dir, ...path.split("/"))).size, 0);
}

/** What the documents of the pack at `dir`, from `document`, reach. */
const reachFrom = (dir: string, document: string): Promise<Reach> => reachOf(document, (path) => readFile(join(dir, ...path.split("/")), "utf8").catch(() => undefined));

/** The texts of the documents at `paths` in `dir` that are there, by path. */
async function documentsOf(dir: string, paths: readonly string[]): Promise<Record<string, string>> {
  const texts = await Promise.all(paths.map(async (path) => [path, await readFile(join(dir, ...path.split("/")), "utf8").catch(() => undefined)] as const));
  return Object.fromEntries(texts.filter((entry): entry is readonly [string, string] => entry[1] !== undefined));
}

/** The version a pack's document exports, if it says one. */
function exportedVersion(text: string): unknown {
  return (mdOf(text).export as { version?: unknown } | undefined)?.version;
}

/** A licence's file name: `LICENSE`, `LICENCE`, `OFL`, `COPYING`, with any extension. */
const LICENCE = /^(licen[cs]e|ofl|copying)([-._].*)?$/i;

/**
 * What the author should check before publishing `pack`, which no host would refuse: each font file with no licence
 * beside it, in its folder, since a font's licence must let a pack ship it (docs/type-params.md, G2).
 *
 * @example
 * await packWarnings(pack); // ["fonts/harbour.woff2 has no licence beside it: put the font's licence (OFL.txt, say) in fonts/"]
 */
export async function packWarnings(pack: Pack): Promise<string[]> {
  if (pack.kind !== "components") return [];
  const warnings: string[] = [];
  for (const path of (await reachFrom(pack.dir, pack.document)).fonts) {
    const folder = posix.dirname(path);
    const at = join(pack.dir, ...folder.split("/"));
    if (!existsSync(at)) continue;
    if (!(await readdir(at)).some((name) => LICENCE.test(name))) warnings.push(`${path} has no licence beside it: put the font's licence (OFL.txt, say) in ${folder === "." ? "the pack's folder" : `${folder}/`}`);
  }
  return [...new Set(warnings)];
}

/**
 * What would go wrong with `pack` once published, each saying what to fix; none when it can be published.
 *
 * @example
 * const problems = await checkPack(await readPack("compmd-packs/mermaid"));
 * // ["index.comp.md's export.version is 0.1.0, package.json's version is 0.2.0: write the same in both"]
 */
export async function checkPack(pack: Pack): Promise<string[]> {
  const { dir, manifest } = pack;
  const problems: string[] = [];
  if (pack.kind === "components") {
    const text = await readFile(join(dir, pack.document), "utf8");
    const version = exportedVersion(text);
    if (version !== manifest.version) problems.push(`${pack.document}'s export.version is ${typeof version === "string" ? version : "missing"}, package.json's version is ${manifest.version}: write the same in both (F022 otherwise)`);
    const published = new Set(await publishedOf(pack));
    const reach = await reachFrom(dir, pack.document);
    // What a host can't serve: an import fails, a data file is null, a font shows its fallback, an image is missing.
    const named = [
      ...reach.imports.map((path) => ({ path, says: `imports ./${path}`, missing: "run the build, or fix the import" })),
      ...reach.data.map((path) => ({ path, says: `reads ./${path}`, missing: "add it, or fix the path" })),
      ...reach.fonts.map((path) => ({ path, says: `declares a font file ./${path}`, missing: "add it, or fix the path" })),
      ...reach.destinations.map((path) => ({ path, says: `links to ./${path}`, missing: "add it, or fix the path" })),
    ];
    for (const { path, says, missing } of named) {
      if (!existsSync(join(dir, ...path.split("/")))) problems.push(`${pack.document} ${says}, which isn't there: ${missing}`);
      else if (!published.has(path)) problems.push(`${pack.document} ${says}, which package.json's \`files\` doesn't publish`);
    }
    for (const path of reach.outside) problems.push(`${pack.document} names ${path}, which leaves the pack or isn't a path in it`);
    const languages = typeof manifest.compmd === "object" ? manifest.compmd.languages : undefined;
    if (languages !== undefined) {
      const documents = await documentsOf(dir, [pack.document, ...reach.imports.filter((path) => path.endsWith(".comp.md"))]);
      problems.push(...(await languageProblems(languages, dir, published, renderedLanguages(documents))));
    }
  } else {
    const library = join(dir, "node_modules", ...pack.library.split("/"), "package.json");
    const version = existsSync(library) ? (JSON.parse(await readFile(library, "utf8")) as { version: string }).version : undefined;
    if (version === undefined) problems.push(`${pack.library} isn't installed: npm install`);
    else if (version !== manifest.version) problems.push(`package.json's version is ${manifest.version}, ${pack.library}'s is ${version}: a library pack's version is its library's, which the build writes`);
  }
  if (pack.kind === "library" && !existsSync(join(dir, ...pack.modules[0].out.split("/")))) problems.push(`${pack.modules[0].out} is missing: run the build`);
  // The build writes each module as one file; another script beside them is a chunk a host couldn't load.
  for (const folder of new Set(pack.modules.map(({ out }) => posix.dirname(out)))) {
    if (!existsSync(join(dir, folder))) continue;
    const modules = new Set(pack.modules.map(({ out }) => out));
    const strays = (await readdir(join(dir, folder))).map((name) => `${folder}/${name}`).filter((path) => /\.m?js$/.test(path) && !modules.has(path));
    if (strays.length > 0) problems.push(`${folder}/ has ${strays.join(", ")} beside the pack's modules: a built module is one file (W121 otherwise); remove what an old build left`);
  }
  const size = await publishedSize(pack);
  if (size > DEFAULT_MAX_PACK_SIZE) problems.push(`the pack is about ${(size / 1_000_000).toFixed(1)} MB unpacked, more than the ${String(DEFAULT_MAX_PACK_SIZE / 1_000_000)} MB a host takes by default`);
  return problems;
}
