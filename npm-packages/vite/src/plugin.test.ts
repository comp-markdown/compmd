import { join, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { AppError } from "./app.ts";
import { everyPack, folderOfVirtualId, outsideTheApp, proxiedOf, twoConfigurations, upToRootOf, virtualIdOf } from "./plugin.ts";

describe("a configuration's virtual module", () => {
  it("keeps the root's id, and gives another folder's its own", () => {
    expect(virtualIdOf("")).toBe("\0virtual:compmd");
    expect(virtualIdOf("website/playground")).toBe("\0virtual:compmd?config=website%2Fplayground");
    expect(folderOfVirtualId(virtualIdOf("website/playground"))).toBe("website/playground");
    expect(folderOfVirtualId("\0virtual:compmd")).toBe("");
    expect(folderOfVirtualId("\0virtual:compmd?other=1")).toBe("");
    expect(folderOfVirtualId("\0virtual:compmdx")).toBeUndefined();
    expect(folderOfVirtualId("/src/main.ts")).toBeUndefined();
  });

  it("is one to a page", () => {
    const one = new Map([[virtualIdOf(""), ["src/main.ts"]]]);
    expect(twoConfigurations("index.html", one)).toBeUndefined();
    const two = new Map([...one, [virtualIdOf("playground"), ["src/helper.ts"]]]);
    expect(twoConfigurations("index.html", two)).toBe(
      "index.html reaches two configurations, which one page never may (docs/vite-per-page-config.md):\n- the root's configuration, imported by src/main.ts\n- playground/'s configuration, imported by src/helper.ts",
    );
  });
});

describe("everyPack", () => {
  it("lists each pack of every configuration once, the first configuration's entry with every one's modules", () => {
    const app = { pack: "@site/app", version: "1.0.0", trust: "trusted" as const, modules: {} };
    const strictApp = { ...app, trust: "strict" as const, modules: { "Stars.svelte.js": "sha384-a" } };
    const mermaid = { pack: "@preview/minimermaid", version: "0.1.0", trust: "strict" as const, modules: { "dist/minimermaid.js": "sha384-b" } };
    expect(everyPack([[app], [strictApp, mermaid]])).toEqual([{ ...app, modules: { "Stars.svelte.js": "sha384-a" } }, mermaid]);
    expect(everyPack([])).toEqual([]);
  });
});

describe("upToRootOf", () => {
  it("finds the bundle from the module's file in a build with a relative base, wherever its page is", () => {
    expect(upToRootOf("build", "./", "assets")).toBe("../");
    expect(upToRootOf("build", "", "static/js")).toBe("../../");
    expect(upToRootOf("build", "./", "")).toBe("");
  });

  it("leaves an absolute base, and development, to the base", () => {
    expect(upToRootOf("build", "/", "assets")).toBeUndefined();
    expect(upToRootOf("build", "/docs/", "assets")).toBeUndefined();
    expect(upToRootOf("serve", "./", "assets")).toBeUndefined();
  });
});

describe("proxiedOf", () => {
  const config = (registries: Record<string, string>, fetch: "host" | "direct" = "host") => ({ registries, fetch });

  it("passes on to the registries of every configuration that fetches through the host", () => {
    const proxied = proxiedOf([
      { folder: "", config: config({ "@kit": "https://npm.kit.example/" }) },
      { folder: "playground", config: config({ "*": "https://registry.npmjs.org/" }) },
      { folder: "readers", config: config({ "@direct": "https://direct.example/" }, "direct") },
    ]);
    expect(proxied).toEqual({ registries: { "@kit": "https://npm.kit.example/", "*": "https://registry.npmjs.org/" }, fetch: "host" });
  });

  it("stops when two configurations send one scope to different registries", () => {
    const configs = [
      { folder: "", config: config({ "*": "https://registry.npmjs.org/" }) },
      { folder: "playground", config: config({ "*": "https://npm.other.example/" }, "direct") },
    ];
    expect(() => proxiedOf(configs)).toThrow(AppError);
    expect(() => proxiedOf(configs)).toThrow("the root's configuration and playground/'s configuration send * to different registries");
  });
});

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
