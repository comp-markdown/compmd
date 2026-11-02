// The publish check, the end of `compmd-pack build` (docs/writing-a-pack.md, decision 4): what a host would refuse,
// or render wrong, found before the pack is published.

import { existsSync, statSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { DEFAULT_MAX_PACK_SIZE } from "@compmd/registry";
import { importedFiles, mdOf, type Pack } from "./pack.ts";

/** The size of `path` in `dir`, a file or a folder of them. */
async function sizeOf(dir: string, path: string): Promise<number> {
  const at = join(dir, ...path.split("/"));
  if (!existsSync(at)) return 0;
  if (!statSync(at).isDirectory()) return statSync(at).size;
  const entries = await readdir(at, { recursive: true, withFileTypes: true });
  return entries.filter((entry) => entry.isFile()).reduce((total, entry) => total + statSync(join(entry.parentPath, entry.name)).size, 0);
}

/** About what npm unpacks of the pack, which a registry's `unpackedSize` says: `files`, the manifest, the README and the licence. */
export async function publishedSize({ dir, manifest }: Pack): Promise<number> {
  const always = (await readdir(dir)).filter((name) => /^(package\.json|readme|license|licence)/i.test(name));
  const paths = [...new Set([...(manifest.files ?? []), ...always])];
  let total = 0;
  for (const path of paths) total += await sizeOf(dir, path.replace(/^\.\//, "").replace(/\/$/, ""));
  return total;
}

/** The version a pack's document exports, if it says one. */
function exportedVersion(text: string): unknown {
  return (mdOf(text).export as { version?: unknown } | undefined)?.version;
}

/**
 * What would go wrong with `pack` once published, each saying what to fix; none when it can be published.
 *
 * @example
 * const problems = await checkPack(await readPack("packs/mermaid"));
 * // ["index.comp.md's export.version is 0.1.0, package.json's version is 0.2.0: write the same in both"]
 */
export async function checkPack(pack: Pack): Promise<string[]> {
  const { dir, manifest } = pack;
  const problems: string[] = [];
  if (pack.kind === "components") {
    const text = await readFile(join(dir, pack.document), "utf8");
    const version = exportedVersion(text);
    if (version !== manifest.version) problems.push(`${pack.document}'s export.version is ${typeof version === "string" ? version : "missing"}, package.json's version is ${manifest.version}: write the same in both (F022 otherwise)`);
    const published = manifest.files ?? [];
    for (const path of importedFiles(pack.document, text)) {
      if (!existsSync(join(dir, ...path.split("/")))) problems.push(`${pack.document} imports ./${path}, which isn't there: run the build, or fix the import`);
      else if (!published.some((entry) => path === posix.normalize(entry) || path.startsWith(`${posix.normalize(entry).replace(/\/$/, "")}/`))) problems.push(`${pack.document} imports ./${path}, which package.json's \`files\` doesn't publish`);
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
