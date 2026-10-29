import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { SHARED } from "./libraries.ts";

/**
 * The libraries components share load through the import map `scripts/libs.ts` builds, never bundled, so each page
 * has one React and one Svelte runtime, the editor's own code included.
 */
function shared(): Plugin {
  return {
    name: "compmd-shared-libraries",
    enforce: "pre",
    resolveId(id) {
      return SHARED.has(id) ? { id, external: true } : undefined;
    },
    transformIndexHtml() {
      const map = readFileSync(join(import.meta.dirname, "public/libs/importmap.json"), "utf8");
      return [{ tag: "script", attrs: { type: "importmap" }, children: map, injectTo: "head-prepend" }];
    },
  };
}

// The frame is a sandboxed iframe with an opaque origin, so its modules load cross-origin.
const cors = { origin: "*" };

export default defineConfig({
  base: "./",
  plugins: [shared(), svelte()],
  build: {
    target: "es2024",
    rolldownOptions: { input: { index: "index.html", frame: "frame.html" } },
    // Svelte's compiler is one chunk, loaded only for a document with Svelte components.
    chunkSizeWarningLimit: 1000,
  },
  worker: { format: "es" },
  server: { cors, fs: { allow: [".."] } },
  preview: { cors },
});
