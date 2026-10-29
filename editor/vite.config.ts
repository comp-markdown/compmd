import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type Plugin } from "vite";
import { SHARED } from "./libraries.ts";

/**
 * The libraries components share load through the import map `scripts/libs.ts` builds, never bundled, so each page
 * has one React and one Svelte runtime, the editor's own code included. A build leaves their imports bare, for the
 * map; the dev server would rewrite a bare import to a URL of its own, so there they go to the URL the map gives.
 */
function shared(): Plugin {
  const map = (): { imports: Record<string, string> } =>
    JSON.parse(readFileSync(join(import.meta.dirname, "public/libs/importmap.json"), "utf8")) as { imports: Record<string, string> };
  let serving = false;
  return {
    name: "compmd-shared-libraries",
    enforce: "pre",
    configResolved(config) {
      serving = config.command === "serve";
    },
    resolveId(id) {
      if (!SHARED.has(id)) return undefined;
      const url = map().imports[id];
      return serving && url ? { id: url.replace(/^\.\//, "/"), external: true } : { id, external: true };
    },
    transformIndexHtml() {
      return [{ tag: "script", attrs: { type: "importmap" }, children: JSON.stringify(map()), injectTo: "head-prepend" }];
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
  // The dev server bundles no shared library ahead either; the compilers it does, for their CommonJS dependencies.
  optimizeDeps: { exclude: [...SHARED], include: ["svelte/compiler", "sucrase"] },
  server: { cors, fs: { allow: [".."] } },
  preview: { cors },
});
