import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Compiles the Svelte binding's runes and the test components.
  plugins: [svelte()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  test: { environment: "happy-dom", include: ["src/**/*.test.ts", "test/**/*.test.ts"] },
});
