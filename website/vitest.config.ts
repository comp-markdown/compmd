import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [svelte()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  server: { fs: { allow: [".."] } },
  test: {
    projects: [
      { extends: true, test: { name: "site", environment: "happy-dom", include: ["src/**/*.test.ts", "build/**/*.test.ts", "snippets/**/*.test.ts"] } },
      // The playground's modules, with `virtual:compmd` of its own configuration (`playground/compmd.config.json`).
      { extends: true, plugins: [compmd()], test: { name: "playground", environment: "happy-dom", include: ["playground/src/**/*.test.ts"] } },
      // The playground built and opened in the installed Chrome, with packs from a local registry.
      { test: { name: "playground-page", environment: "node", include: ["playground/test/**/*.test.ts"], testTimeout: 60_000, hookTimeout: 120_000 } },
    ],
  },
});
