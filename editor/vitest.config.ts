import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Compiles the render package's Svelte binding.
  plugins: [svelte()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  server: { fs: { allow: [".."] } },
  test: { environment: "happy-dom", include: ["src/**/*.test.ts"] },
});
