// `vite build` writes the component as one ES module, `dist/__component__.js`, with every library it imports bundled
// in, but the runtimes the sandbox's frames provide (spec §7.6), which it adds to `peerDependencies`, and the library
// packs it names there, which packs share. `vite` opens the test page, `demo/`, which uses the pack as a host does, in
// the strict sandbox, and builds the component again when its source changes.
import { compmd } from "@compmd/vite";
import { RUNTIMES } from "@compmd/sandbox/build";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { build, defineConfig, type BuildEnvironmentOptions, type Plugin } from "vite";

const root = import.meta.dirname;

/** The runtimes' import names: the frames' import map gives these, one copy for every component. */
const RUNTIME_IMPORTS = new Set(RUNTIMES.flatMap((runtime) => runtime.imports));

/** The library packs the pack shares, which it names in `peerDependencies`: a host gives one copy to every pack. */
const SHARED = Object.keys((JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as { peerDependencies?: Record<string, string> }).peerDependencies ?? {});

/** Whether `id` stays an import of the built module: a runtime, or a shared library or one of its paths. */
const external = (id: string): boolean => RUNTIME_IMPORTS.has(id) || SHARED.some((name) => id === name || id.startsWith(`${name}/`));

/**
 * Adds the runtimes the built module imports to `peerDependencies`, at the version it was built with, so a host that
 * has an older one says so when it resolves the pack (W121) instead of failing at render.
 */
function peers(): Plugin {
  return {
    name: "peers",
    generateBundle(_options, bundle) {
      const imported = Object.values(bundle).flatMap((chunk) => (chunk.type === "chunk" ? chunk.imports : []));
      const runtimes = RUNTIMES.filter((runtime) => imported.some((name) => runtime.imports.includes(name)));
      const manifest = JSON.parse(readFileSync(join(root, "package.json"), "utf8")) as { peerDependencies?: Record<string, string> };
      const missing = runtimes.filter((runtime) => !manifest.peerDependencies?.[runtime.name]);
      if (missing.length === 0) return;
      const peerDependencies = { ...manifest.peerDependencies };
      for (const runtime of missing) {
        const { version } = JSON.parse(readFileSync(join(root, "node_modules", runtime.name, "package.json"), "utf8")) as { version: string };
        peerDependencies[runtime.name] = `^${version}`;
      }
      writeFileSync(join(root, "package.json"), `${JSON.stringify({ ...manifest, peerDependencies }, null, 2)}\n`);
      this.warn(`added ${missing.map((runtime) => runtime.name).join(", ")} to package.json's peerDependencies`);
    },
  };
}

const library: BuildEnvironmentOptions = {
  lib: { entry: join(root, "src/__component__.ts"), formats: ["es"], fileName: () => "__component__.js" },
  outDir: join(root, "dist"),
  target: "es2024",
  minify: true,
  // Two copies of React break its hooks, so a built module never bundles a runtime. A host may load the module as text,
  // where a chunk has no path to load from, so a library that loads parts of itself on demand stays in the one file.
  rolldownOptions: { external, plugins: [peers()], output: { codeSplitting: false } },
};

/** Builds the component again when a file in `src/` changes; the test page then loads it again. */
function rebuild(): Plugin {
  return {
    name: "rebuild",
    configureServer(server) {
      server.watcher.add([join(root, "src"), join(root, "dist")]);
      server.watcher.on("change", (file) => {
        if (file.startsWith(join(root, "src")) && !file.endsWith(".test.ts")) void build({ configFile: false, root, logLevel: "warn", build: library });
      });
    },
  };
}

// Vitest runs the component's tests alone: the test page and its sandbox need the built module, which tests don't.
export default defineConfig(({ command, mode }) => (mode === "test" ? {} : command === "build" ? { build: library } : { root: join(root, "demo"), plugins: [compmd(), rebuild()] }));
