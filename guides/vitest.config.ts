import { defineConfig } from "vitest/config";

export default defineConfig({
  // The chapters against their examples, and the examples built and opened in Chrome and WebKit.
  test: { environment: "node", include: ["test/**/*.test.ts"], testTimeout: 60_000, hookTimeout: 120_000 },
});
