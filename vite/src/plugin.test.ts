import { join, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { outsideTheApp } from "./plugin.ts";

describe("outsideTheApp", () => {
  const root = resolve("/apps/notes");

  it("keeps a bundle written inside the app out of the dev server's watch and its dependency scan", () => {
    expect(outsideTheApp({ root }, "src-tauri/compmd")).toEqual({
      server: { watch: { ignored: [`${join(root, "src-tauri/compmd").split("\\").join("/")}/**`] } },
      optimizeDeps: { entries: ["**/*.html", "!**/node_modules/**", "!**/__tests__/**", "!**/coverage/**", "!src-tauri/compmd/**"] },
    });
  });

  it("leaves the app's own entries as they are", () => {
    expect(outsideTheApp({ root, optimizeDeps: { entries: ["index.html"] } }, "src-tauri/compmd")?.optimizeDeps).toBeUndefined();
  });

  it("changes nothing for a web app, or a bundle outside the app", () => {
    expect(outsideTheApp({ root }, undefined)).toBeUndefined();
    expect(outsideTheApp({ root }, "../desktop/compmd")).toBeUndefined();
    expect(outsideTheApp({ root }, ".")).toBeUndefined();
  });
});
