import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // Reading, rendering and writing a blog, in Node.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts"] } },
      // A blog served and opened in Chrome and WebKit through Playwright.
      { test: { name: "browser", environment: "node", include: ["test/**/*.test.ts"], testTimeout: 60_000, hookTimeout: 120_000 } },
    ],
  },
});
