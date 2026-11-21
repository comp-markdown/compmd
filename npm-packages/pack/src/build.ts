// `compmd-pack build` (docs/writing-a-pack.md, decisions 2 and 4): each module the pack publishes, as one ES module
// with every library bundled in but the runtimes the sandbox's frames provide (spec §7.6) and the library packs the
// pack names in `peerDependencies`, which packs share; then the publish check.

import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { RUNTIMES } from "@compmd/sandbox/build";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { build, type InlineConfig, type Plugin } from "vite";
import { PackError, type Manifest, type Pack } from "./pack.ts";
import { readWrap } from "./wrap/config.ts";
import { generateSignature } from "./wrap/generate.ts";
import { installedPackage } from "./wrap/installed.ts";
import { risksOf } from "./wrap/risks.ts";

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
      // Found as Node finds them: in the pack's `node_modules`, or a workspace's above it.
      const versionOf = (name: string) => installedPackage(pack.dir, name)?.version ?? manifestOf(join(pack.dir, "node_modules", name)).version;
      for (const runtime of added) peerDependencies[runtime.name] = `^${versionOf(runtime.name)}`;
      const version = pack.kind === "library" ? versionOf(pack.library) : written.version;
      const changes = [...(written.version === version ? [] : [`version ${version}, ${pack.kind === "library" ? pack.library : ""}'s`]), ...added.map((runtime) => `${runtime.name} in peerDependencies`)];
      if (changes.length === 0) return;
      writeFileSync(join(pack.dir, "package.json"), `${JSON.stringify({ ...written, version, ...(added.length > 0 ? { peerDependencies } : {}) }, null, 2)}\n`);
      this.warn(`wrote into package.json: ${changes.join("; ")}`);
    },
  };
}

/** How a module is built: Vite's log level, and whether it compiles a Svelte package's sources. */
export interface ModuleOptions {
  logLevel?: InlineConfig["logLevel"];
  /**
   * A wrapped Svelte package's `.svelte` sources, compiled with the Svelte the frames provide, their CSS injected by the
   * module, since a frame loads nothing else (docs/wrapping-an-npm-package.md, decision 10).
   */
  svelte?: boolean;
}

/** Vite's configuration for one of the pack's modules. */
export function moduleConfig(pack: Pack, module: Pack["modules"][number], options: ModuleOptions = {}): InlineConfig {
  const shared = Object.keys(pack.manifest.peerDependencies ?? {});
  const out = module.out.split("/");
  return {
    configFile: false,
    root: pack.dir,
    plugins: options.svelte ? [svelte({ configFile: false, emitCss: false, compilerOptions: { css: "injected" } })] : [],
    logLevel: options.logLevel ?? "warn",
    // What is published, whatever NODE_ENV the build runs under (a test runner sets `test`): the production JSX, and
    // the production branches of the libraries it bundles, with no `process` left, which a frame doesn't have.
    mode: "production",
    define: { "process.env.NODE_ENV": JSON.stringify("production") },
    // Vite takes development JSX from NODE_ENV itself, whatever the mode.
    oxc: { jsx: { runtime: "automatic", development: false } },
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
      // Minified in full: Vite's `minify` keeps an ES library's whitespace, and the comments naming the files the module
      // was built from, which hold the author's paths. A frame loads the module as it is, with no bundler after it that
      // would need the annotations kept for it.
      rolldownOptions: { external: (id: string) => isExternal(id, shared), plugins: [manifest(pack)], output: { codeSplitting: false, minify: true } },
    },
  };
}

/**
 * Builds every module `pack` publishes. A pack that wraps a package (`compmd.wrap.json`) has its signature written
 * first, from the package's types, and its modules checked for what a strict frame refuses after.
 *
 * @returns What the author should read: what the wrap wrote and changed, and the warnings.
 * @throws PackError when the wrap can't write its signature, or a param that went is still named.
 * @example
 * await buildModules(await readPack("compmd-packs/mermaid"));
 */
export async function buildModules(pack: Pack, options: Omit<ModuleOptions, "svelte"> = {}): Promise<string[]> {
  const wrap = pack.kind === "components" ? readWrap(pack.dir) : undefined;
  const notes: string[] = [];
  if (wrap && wrap.kind !== "library" && pack.kind === "components") {
    const report = generateSignature(pack.dir, pack.document, wrap);
    if (report.errors.length > 0) throw new PackError(report.errors.join("\n"));
    notes.push(...report.notes);
  }
  for (const module of pack.modules) await build(moduleConfig(pack, module, { ...options, svelte: wrap?.kind === "svelte" }));
  if (wrap) for (const module of pack.modules) notes.push(...risksOf(module.out, readFileSync(join(pack.dir, ...module.out.split("/")), "utf8")));
  return notes;
}
