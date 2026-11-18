import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Compiles the render package's Svelte binding; `virtual:compmd` gives the documents the playground's packs.
  plugins: [svelte(), compmd()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  server: { fs: { allow: [".."] } },
  test: {
    projects: [
      { extends: true, test: { name: "unit", environment: "happy-dom", include: ["src/**/*.test.ts"] } },
      // The playground served as in development and opened in the installed Chrome, with packs from a local registry.
      { test: { name: "page", environment: "node", include: ["test/**/*.test.ts"], testTimeout: 60_000, hookTimeout: 120_000 } },
    ],
  },
});
