import { defineConfig } from "vitest/config";

export default defineConfig({
  // The chapters against their examples, and the examples built and opened in Chrome and WebKit.
  // `test/.out` holds what the tests build: a copy of the example pack, with tests of its own that its own command runs.
  test: { environment: "node", include: ["test/**/*.test.ts"], exclude: ["test/.out/**"], testTimeout: 60_000, hookTimeout: 120_000 },
});
