// `compmd-pack build` (docs/writing-a-pack.md, decisions 2 and 4): each module the pack publishes, as one ES module
// with every library bundled in but the runtimes the sandbox's frames provide (spec §7.6) and the library packs the
// pack names in `peerDependencies`, which packs share; then the publish check.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { RUNTIMES } from "@compmd/sandbox/build";
import { build, type InlineConfig, type Plugin } from "vite";
import type { Manifest, Pack } from "./pack.ts";

/** The runtimes' import names: the frames' import map gives these, one copy for every component. */
const RUNTIME_IMPORTS = new Set(RUNTIMES.flatMap((runtime) => runtime.imports));

const manifestOf = (dir: string): Manifest => JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as Manifest;

/** Whether `id` stays an import of a built module: a runtime, or a library pack it shares, or one of its paths. */
export function isExternal(id: string, shared: readonly string[]): boolean {
  return RUNTIME_IMPORTS.has(id) || shared.some((name) => id === name || id.startsWith(`${name}/`));
}

/**
 * Writes into `package.json` what the built modules make true: the runtimes they import as `peerDependencies`, at
 * the versions they were built with, so a host with an older one says so when it resolves the pack (W121) instead of
 * failing at render; and a library pack's version, which is its library's.
 */
function manifest(pack: Pack): Plugin {
  return {
    name: "compmd-pack:manifest",
    generateBundle(_options, bundle) {
      const imported = Object.values(bundle).flatMap((chunk) => (chunk.type === "chunk" ? chunk.imports : []));
      const written = manifestOf(pack.dir);
      const peerDependencies = { ...written.peerDependencies };
      const added = RUNTIMES.filter((runtime) => imported.some((name) => runtime.imports.includes(name)) && !peerDependencies[runtime.name]);
      for (const runtime of added) peerDependencies[runtime.name] = `^${manifestOf(join(pack.dir, "node_modules", runtime.name)).version}`;
      const version = pack.kind === "library" ? manifestOf(join(pack.dir, "node_modules", pack.library)).version : written.version;
      const changes = [...(written.version === version ? [] : [`version ${version}, ${pack.kind === "library" ? pack.library : ""}'s`]), ...added.map((runtime) => `${runtime.name} in peerDependencies`)];
      if (changes.length === 0) return;
      writeFileSync(join(pack.dir, "package.json"), `${JSON.stringify({ ...written, version, ...(added.length > 0 ? { peerDependencies } : {}) }, null, 2)}\n`);
      this.warn(`wrote into package.json: ${changes.join("; ")}`);
    },
  };
}

/** Vite's configuration for one of the pack's modules. */
export function moduleConfig(pack: Pack, module: Pack["modules"][number], options: { logLevel?: InlineConfig["logLevel"] } = {}): InlineConfig {
  const shared = Object.keys(pack.manifest.peerDependencies ?? {});
  const out = module.out.split("/");
  return {
    configFile: false,
    root: pack.dir,
    logLevel: options.logLevel ?? "warn",
    build: {
      lib: { entry: join(pack.dir, ...module.source.split("/")), formats: ["es"], fileName: () => out.at(-1) ?? "index.js" },
      outDir: join(pack.dir, ...out.slice(0, -1)),
      // Modules share `dist/`: each build writes its own file, and leaves the others'.
      emptyOutDir: false,
      target: "es2024",
      minify: true,
      // The licences of what it bundles, which a module built from libraries publishes with them; one file per module.
      license: { fileName: pack.modules.length === 1 ? "LICENSES.md" : `${(out.at(-1) ?? "index.js").replace(/\.js$/, "")}.LICENSES.md` },
      // Two copies of React break its hooks, so a built module never bundles a runtime. A host may load the module
      // as text, where a chunk has no path to load from, so a library that loads parts of itself on demand stays in
      // the one file.
      rolldownOptions: { external: (id: string) => isExternal(id, shared), plugins: [manifest(pack)], output: { codeSplitting: false } },
    },
  };
}

/**
 * Builds every module `pack` publishes.
 *
 * @example
 * await buildModules(await readPack("packs/mermaid"));
 */
export async function buildModules(pack: Pack, options: { logLevel?: InlineConfig["logLevel"] } = {}): Promise<void> {
  for (const module of pack.modules) await build(moduleConfig(pack, module, options));
}
