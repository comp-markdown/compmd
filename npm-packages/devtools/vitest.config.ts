import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [svelte()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  test: { environment: "happy-dom", include: ["src/**/*.test.ts"] },
});
