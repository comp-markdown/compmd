// `vite build` writes the library `__library__` as one ES module, `dist/index.js`, with everything it imports bundled
// in, but the runtimes the sandbox's frames provide (spec §7.6) and the library packs it names in `peerDependencies`.
// The pack's version is the library's, and the runtimes it imports are its peers: the build writes both into
// `package.json`.
import { RUNTIMES } from "@compmd/sandbox/build";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";

const root = import.meta.dirname;
const LIBRARY = "__library__";

interface Manifest {
  version: string;
  peerDependencies?: Record<string, string>;
}

/** The `package.json` in `dir`. */
const manifestOf = (dir: string): Manifest => JSON.parse(readFileSync(join(dir, "package.json"), "utf8")) as Manifest;

/** The runtimes' import names: the frames' import map gives these, one copy for every component. */
const RUNTIME_IMPORTS = new Set(RUNTIMES.flatMap((runtime) => runtime.imports));

/** The library packs this one uses, which it names in `peerDependencies`. */
const SHARED = Object.keys(manifestOf(root).peerDependencies ?? {});

/** Whether `id` stays an import of the built module: a runtime, or a shared library or one of its paths. */
const external = (id: string): boolean => RUNTIME_IMPORTS.has(id) || SHARED.some((name) => id === name || id.startsWith(`${name}/`));

/**
 * Writes into `package.json` the installed library's version as the pack's, and the runtimes the module imports as
 * `peerDependencies`, at the versions it was built with, so a host with an older one says so (W121).
 */
function manifest(): Plugin {
  return {
    name: "manifest",
    generateBundle(_options, bundle) {
      const imported = Object.values(bundle).flatMap((chunk) => (chunk.type === "chunk" ? chunk.imports : []));
      const written = manifestOf(root);
      const version = manifestOf(join(root, "node_modules", LIBRARY)).version;
      const peerDependencies = { ...written.peerDependencies };
      const added = RUNTIMES.filter((runtime) => imported.some((name) => runtime.imports.includes(name)) && !peerDependencies[runtime.name]);
      for (const runtime of added) peerDependencies[runtime.name] = `^${manifestOf(join(root, "node_modules", runtime.name)).version}`;
      const changes = [...(written.version === version ? [] : [`version ${version}, ${LIBRARY}'s`]), ...added.map((runtime) => `${runtime.name} in peerDependencies`)];
      if (changes.length === 0) return;
      writeFileSync(join(root, "package.json"), `${JSON.stringify({ ...written, version, ...(added.length > 0 ? { peerDependencies } : {}) }, null, 2)}\n`);
      this.warn(`wrote into package.json: ${changes.join("; ")}`);
    },
  };
}

export default defineConfig({
  build: {
    lib: { entry: join(root, "src/index.ts"), formats: ["es"], fileName: () => "index.js" },
    outDir: join(root, "dist"),
    target: "es2024",
    minify: true,
    // A host loads the module as text, where a chunk has no path to load from, so it stays one file.
    rolldownOptions: { external, plugins: [manifest()], output: { codeSplitting: false } },
  },
});
