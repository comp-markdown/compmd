// Builds what the package ships for its frame documents into `dist/frames/`: the frame runtime, with the compilers and
// Temml as chunks it loads when needed, and the runtimes every frame shares through its import map. `buildFrames`
// then writes the documents for a host's origin.

import { svelte } from "@sveltejs/vite-plugin-svelte";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { build } from "vite";
import { buildLibraries, RUNTIMES, SHARED } from "../src/build/libraries.ts";

const ROOT = join(import.meta.dirname, "..");
// Apart from `dist/frame/`, which holds the runtime's modules as the TypeScript build gives them.
const OUT = join(ROOT, "dist/frames");

await rm(OUT, { recursive: true, force: true });
await build({
  configFile: false,
  root: ROOT,
  // Its chunks load next to it, wherever a host serves the frame documents.
  base: "./",
  logLevel: "warn",
  // Compiles `@compmd/render`'s Svelte binding, whose runes are Svelte's.
  plugins: [svelte()],
  resolve: { conditions: ["browser"] },
  build: {
    target: "es2024",
    outDir: OUT,
    emptyOutDir: false,
    copyPublicDir: false,
    // Svelte's compiler is one chunk, loaded only by the compile frame for a Svelte component.
    chunkSizeWarningLimit: 1000,
    rolldownOptions: {
      input: { runtime: join(ROOT, "src/frame/runtime.ts") },
      // The runtimes stay imports, for the frame's import map: one React and one Svelte, whatever loads them.
      external: [...SHARED],
      output: { format: "es", entryFileNames: "[name].js", chunkFileNames: "chunks/[name]-[hash].js" },
    },
  },
});
await buildLibraries(RUNTIMES, { root: ROOT, out: join(OUT, "libs") });
console.log(`Frame runtime and ${RUNTIMES.map((library) => library.name).join(", ")} in dist/frames/`);
