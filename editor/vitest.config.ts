import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Compiles the render package's Svelte binding; `virtual:compmd` gives the documents the playground's packs.
  plugins: [svelte(), compmd()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  server: { fs: { allow: [".."] } },
  test: { environment: "happy-dom", include: ["src/**/*.test.ts"] },
});
