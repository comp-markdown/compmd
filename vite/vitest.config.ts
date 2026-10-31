import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // The configuration, finding packs and the plugin's own parts, in Node.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts"] } },
      // The example app, built with the plugin and served, in Chrome and WebKit through Playwright.
      { test: { name: "example", environment: "node", include: ["test/**/*.test.ts"], testTimeout: 30_000 } },
    ],
  },
});
