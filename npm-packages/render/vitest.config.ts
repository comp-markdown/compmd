import { svelte } from "@sveltejs/vite-plugin-svelte";
import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

export default defineConfig({
  // Compiles the Svelte binding's runes and the test components.
  plugins: [svelte()],
  // Svelte's browser build, which `mount` needs.
  resolve: { conditions: ["browser"] },
  test: {
    projects: [
      { extends: true, test: { name: "unit", environment: "happy-dom", include: ["src/**/*.test.ts", "test/**/*.test.ts"], exclude: ["test/browser/**"] } },
      // What only a browser's layout says: how a component's box sizes what it holds. Chrome as installed, and WebKit.
      {
        extends: true,
        test: {
          name: "browser",
          include: ["test/browser/**/*.test.ts"],
          browser: {
            enabled: true,
            headless: true,
            screenshotFailures: false,
            instances: [
              { browser: "chromium", provider: playwright({ launchOptions: { channel: "chrome" } }) },
              { browser: "webkit", provider: playwright() },
            ],
          },
        },
      },
    ],
  },
});
