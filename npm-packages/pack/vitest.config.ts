import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // The command's parts, in Node, on packs written into temporary folders.
      { test: { name: "node", environment: "node", include: ["src/**/*.test.ts"], exclude: ["src/component.test.ts", "src/testing.test.ts"], testTimeout: 60_000 } },
      // The component helpers and the testing kit, in happy-dom, as a pack's own tests run them.
      { test: { name: "dom", environment: "happy-dom", include: ["src/component.test.ts", "src/testing.test.ts"] } },
    ],
  },
});
