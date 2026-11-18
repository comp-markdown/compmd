import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // Writing a pack, in Node.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts"] } },
      // The template's components, in happy-dom, as `compmd-pack test` runs a written pack's tests.
      { test: { name: "template", environment: "happy-dom", include: ["template/*/src/*.test.ts"] } },
      // A pack written, built and opened in its test page, in Chrome and WebKit through Playwright. The packs written
      // run their own tests, in their folders.
      { test: { name: "pack", environment: "node", include: ["test/**/*.test.ts"], exclude: ["test/.out/**"], testTimeout: 60_000, hookTimeout: 120_000 } },
    ],
  },
});
