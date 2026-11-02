import { readFileSync } from "node:fs";
import { join } from "node:path";
import { Ajv2020 } from "ajv/dist/2020.js";
import { describe, expect, it } from "vitest";
import { readConfig } from "./config.ts";

const schema = JSON.parse(readFileSync(join(import.meta.dirname, "../compmd.config.schema.json"), "utf8")) as object;
const validate = new Ajv2020({ strict: true }).compile(schema);

/** A configuration with every key, which `readConfig` takes. */
const EVERY_KEY = {
  $schema: "./node_modules/@compmd/vite/compmd.config.schema.json",
  packs: {
    "@kit/mermaid": { version: "^1.0.0" },
    "@acme/maps": { trust: "permissive", network: ["https://tiles.example.com"] },
    "@acme/reviews": { trust: "permissive", host: true },
    "./components": { trust: "trusted", ambient: true },
  },
  imports: "listed",
  deny: [{ pack: "@bad/pack", version: "<2", reason: "a flaw found in 1.x" }],
  maxPackSize: "5 MB",
  libraries: ["d3", { name: "chart.js", imports: ["chart.js/auto"] }],
  registries: { "@kit": "https://npm.kit.example/", "*": "https://registry.npmjs.org/" },
  fetch: "direct",
};

describe("compmd.config.schema.json", () => {
  it("takes what readConfig takes", () => {
    expect(() => readConfig(EVERY_KEY)).not.toThrow();
    expect(validate(EVERY_KEY), JSON.stringify(validate.errors)).toBe(true);
    expect(validate({ maxPackSize: 1200 })).toBe(true);
  });

  it("refuses the shapes readConfig refuses", () => {
    for (const wrong of [
      { pack: {} },
      { packs: { "acme-maps": {} } },
      { packs: { "@a/b": { trsut: "strict" } } },
      { packs: { "@a/b": { trust: "open" } } },
      { packs: { "@a/b": { trust: "permissive", network: ["https://*.example.com"] } } },
      { imports: "some" },
      { deny: [{ pack: "@a/b" }] },
      { maxPackSize: "large" },
      { libraries: [{ name: "d3" }] },
      { registries: { kit: "https://npm.kit.example/" } },
      { registries: { "*": "/compmd/registry/" } },
      { registries: { "*": "https://token@npm.kit.example/" } },
      { registries: ["https://registry.npmjs.org/"] },
      { fetch: "proxy" },
    ]) {
      expect(() => readConfig(wrong), JSON.stringify(wrong)).toThrow();
      expect(validate(wrong), JSON.stringify(wrong)).toBe(false);
    }
  });
});
