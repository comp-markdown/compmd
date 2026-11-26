import { defineConfig } from "vitest/config";

export default defineConfig({
  // A build and two browsers per file: their load, not a feature, takes the time (docs/code/playwright.md).
  test: { environment: "node", include: ["test/**/*.test.ts"], testTimeout: 60_000, hookTimeout: 180_000 },
});
