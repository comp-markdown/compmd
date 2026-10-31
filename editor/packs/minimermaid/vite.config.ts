// `vite build` writes the component as one ES module, `dist/minimermaid.js`, with every library it imports bundled
// in, but the runtimes the sandbox's frames provide. `vite` opens the test page, `demo/`, which uses the pack as a host
// does, in the strict sandbox, and builds the component again when its source changes.
import { compmd } from "@compmd/vite";
import { join } from "node:path";
import { build, defineConfig, type BuildEnvironmentOptions, type Plugin } from "vite";

const root = import.meta.dirname;

const library: BuildEnvironmentOptions = {
  lib: { entry: join(root, "src/minimermaid.ts"), formats: ["es"], fileName: () => "minimermaid.js" },
  outDir: join(root, "dist"),
  target: "es2024",
  minify: true,
  // The frames' import map gives these, one copy for every component: two copies of React break its hooks.
  rolldownOptions: { external: /^(react|react-dom|svelte)(\/|$)/ },
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
