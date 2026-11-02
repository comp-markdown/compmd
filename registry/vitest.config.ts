import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // The reader, the cache, the resolver and the proxy, against a local registry.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts"] } },
      // What resolving rests on is the browser's: DecompressionStream, crypto.subtle, Cache Storage, cross-origin
      // fetches. Chrome as installed, and WebKit.
      {
        test: {
          name: "browser",
          include: ["test/**/*.test.ts"],
          globalSetup: ["test/setup.ts"],
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
