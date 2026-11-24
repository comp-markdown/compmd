// What a pack is, read from its folder (docs/writing-a-pack.md, decision 1): its manifest, and the modules it builds.
// A pack of components names its document in `package.json`'s `compmd`; the document imports each component's built
// module, `./dist/<name>.js`, built from `src/<name>.ts` (or `.tsx`, `.js`, `.jsx`). A library pack names its module
// in `compmd.library`, built from `src/index.ts`, which re-exports the library.

import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join, posix } from "node:path";
import { documentOf, PackError, referencesOf } from "@compmd/vite/pack-folder";

export { PackError };

/** The parts of `package.json` the command reads. */
export interface Manifest {
  name: string;
  version: string;
  compmd?: string | { library?: string };
  files?: string[];
  peerDependencies?: Record<string, string>;
}

/** One module a pack builds: its source, and where it is written, both relative to the pack's folder. */
export interface Module {
  source: string;
  out: string;
}

/** A pack, as the command builds, checks and serves it. */
export type Pack =
  | { kind: "components"; dir: string; manifest: Manifest; document: string; modules: Module[] }
  | { kind: "library"; dir: string; manifest: Manifest; library: string; modules: [Module] };

const SOURCES = [".ts", ".tsx", ".js", ".jsx"];
const BUILT = /^dist\/[^/]+\.js$/;

/** The source a built module at `out` (`dist/<name>.js`) is built from. */
function sourceOf(dir: string, out: string): string {
  const stem = `src/${posix.basename(out, ".js")}`;
  const found = SOURCES.map((extension) => `${stem}${extension}`).find((source) => existsSync(join(dir, ...source.split("/"))));
  if (!found) throw new PackError(`${out} is imported by the pack's document, but there is no ${stem}.ts (or .tsx, .js, .jsx) to build it from`);
  return found;
}

/** The library a library pack's `src/index.ts` re-exports: its one bare import, comments aside. */
function libraryOf(source: string): string | undefined {
  const code = source.replace(/\/\*[\s\S]*?\*\/|\/\/.*$/gm, "");
  const bare = [...code.matchAll(/\bfrom\s*["']([^"'.][^"']*)["']/g)].map((match) => match[1] ?? "");
  const names = [...new Set(bare.map((specifier) => specifier.split("/").slice(0, specifier.startsWith("@") ? 2 : 1).join("/")))];
  return names.length === 1 ? names[0] : undefined;
}

/**
 * The pack in `dir`.
 *
 * @throws PackError when `package.json` names no document or library, or a module has nothing to build it from.
 * @example
 * const pack = await readPack("compmd-packs/mermaid");
 * pack.modules; // [{ source: "src/mermaid.ts", out: "dist/mermaid.js" }]
 */
export async function readPack(dir: string): Promise<Pack> {
  const manifest = JSON.parse(await readFile(join(dir, "package.json"), "utf8")) as Manifest;
  const { compmd } = manifest;
  if (typeof compmd === "object" && typeof compmd.library === "string") {
    const source = "src/index.ts";
    if (!existsSync(join(dir, source))) throw new PackError(`a library pack builds ${compmd.library} from ${source}, which isn't there`);
    const library = libraryOf(await readFile(join(dir, source), "utf8"));
    if (!library) throw new PackError(`${source} should re-export one npm package, the library: export * from "three";`);
    return { kind: "library", dir, manifest, library, modules: [{ source, out: compmd.library }] };
  }
  if (typeof compmd !== "string") throw new PackError('package.json\'s `compmd` names the pack\'s document ("./index.comp.md") or, for a library pack, its module ({ "library": "dist/index.js" })');
  const document = documentOf(manifest);
  if (!existsSync(join(dir, document))) throw new PackError(`package.json's \`compmd\` is ${compmd}, which isn't there`);
  const built = referencesOf(document, await readFile(join(dir, document), "utf8")).imports.filter((path) => BUILT.test(path));
  return { kind: "components", dir, manifest, document, modules: built.map((out) => ({ source: sourceOf(dir, out), out })) };
}
