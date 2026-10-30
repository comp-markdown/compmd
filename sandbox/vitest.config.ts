import { playwright } from "@vitest/browser-playwright";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      // The protocol and what needs no DOM.
      { test: { name: "unit", environment: "node", include: ["src/**/*.test.ts"] } },
      // What the sandbox rests on is the browser's: frames, origins, policies. Chrome as installed, and WebKit, the
      // engine of an application's web view on macOS and Linux.
      {
        test: {
          name: "browser",
          include: ["test/**/*.test.ts"],
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
