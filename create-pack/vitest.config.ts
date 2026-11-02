import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // Writing a pack, in Node; and the template's components, in happy-dom, as a written pack's tests run them.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts", "template/*/src/*.test.ts"] } },
      // A pack written, built and opened in its test page, in Chrome and WebKit through Playwright. The packs written
      // run their own tests, in their folders.
      { test: { name: "pack", environment: "node", include: ["test/**/*.test.ts"], exclude: ["test/.out/**"], testTimeout: 60_000, hookTimeout: 120_000 } },
    ],
  },
});
