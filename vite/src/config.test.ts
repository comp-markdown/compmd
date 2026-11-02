import { describe, expect, it } from "vitest";
import { ConfigError, isFolder, policyOf, readConfig } from "./config.ts";

/** The problems `value` has, as `readConfig` lists them. */
function problemsOf(value: unknown): readonly string[] {
  try {
    readConfig(value);
  } catch (error) {
    if (error instanceof ConfigError) return error.problems;
    throw error;
  }
  return [];
}

describe("readConfig", () => {
  it("fills in every default", () => {
    const config = readConfig({});
    expect(config).toEqual({ packs: new Map(), imports: "any", deny: [], libraries: [], registries: {}, fetch: "host" });
  });

  it("reads the registries by scope, each URL ending in `/`, and who fetches", () => {
    const config = readConfig({ registries: { "@kit": "https://npm.kit.example/npm", "*": "https://registry.npmjs.org/" }, fetch: "direct" });
    expect(config.registries).toEqual({ "@kit": "https://npm.kit.example/npm/", "*": "https://registry.npmjs.org/" });
    expect(config.fetch).toBe("direct");
    expect(readConfig({ registries: { "*": "http://localhost:4873" } }).registries).toEqual({ "*": "http://localhost:4873/" });
  });

  it("takes registries named in full, on https, with nothing secret in their URLs", () => {
    const registry = (url: unknown) => problemsOf({ registries: { "*": url } });
    expect(registry("/compmd/registry/")).toEqual(['registries["*"]: `/compmd/registry/` isn\'t a URL: a registry is named in full, such as `https://registry.npmjs.org/`']);
    expect(registry("http://npm.kit.example/")).toEqual(['registries["*"]: `http://npm.kit.example/` must be `https:` (`http:` only for localhost)']);
    expect(registry("https://me:token@npm.kit.example/")).toEqual([
      'registries["*"]: `https://me:token@npm.kit.example/` must have no credentials, query or fragment: a registry\'s token belongs to the host\'s proxy',
    ]);
    expect(registry("https://npm.kit.example/?token=x")).toEqual([
      'registries["*"]: `https://npm.kit.example/?token=x` must have no credentials, query or fragment: a registry\'s token belongs to the host\'s proxy',
    ]);
    expect(registry(42)).toEqual(['registries["*"]: a registry\'s URL']);
    expect(problemsOf({ registries: { kit: "https://npm.kit.example/" } })).toEqual(['registries["kit"]: a scope, such as `@kit`, or `*` for every other scope']);
    expect(problemsOf({ registries: ["https://registry.npmjs.org/"], fetch: "proxy" })).toEqual([
      "registries: an object, by scope (`@kit`), with `*` for every other scope",
      "fetch: `host` or `direct`",
    ]);
  });

  it("reads each kind of pack, with its trust", () => {
    const config = readConfig({
      $schema: "./node_modules/@compmd/vite/compmd.config.schema.json",
      packs: {
        "@kit/mermaid": {},
        "@acme/maps": { trust: "permissive", network: ["https://tiles.example.com/", "http://localhost:8080"], version: "^3.0.0" },
        "@acme/reviews": { trust: "permissive", host: true },
        "./components": { trust: "trusted", ambient: true },
      },
      imports: "listed",
      deny: [{ pack: "@bad/pack", version: "<2", reason: "a flaw found in 1.x" }, { pack: "@games/kit", reason: "not for school" }],
      maxPackSize: "5 MB",
      libraries: ["d3", { name: "chart.js", imports: ["chart.js", "chart.js/auto"] }],
    });
    expect(config.packs.get("@kit/mermaid")).toEqual({ trust: { level: "strict" }, ambient: false });
    expect(config.packs.get("@acme/maps")).toEqual({
      trust: { level: "permissive", network: ["https://tiles.example.com", "http://localhost:8080"], host: false },
      ambient: false,
      version: "^3.0.0",
    });
    expect(config.packs.get("@acme/reviews")?.trust).toEqual({ level: "permissive", network: [], host: true });
    expect(config.packs.get("./components")).toEqual({ trust: { level: "trusted" }, ambient: true });
    expect(config.imports).toBe("listed");
    expect(config.deny).toEqual([{ pack: "@bad/pack", version: "<2", reason: "a flaw found in 1.x" }, { pack: "@games/kit", reason: "not for school" }]);
    expect(config.maxPackSize).toBe(5_000_000);
    expect(config.libraries).toEqual([
      { name: "d3", imports: ["d3"] },
      { name: "chart.js", imports: ["chart.js", "chart.js/auto"] },
    ]);
  });

  it("takes a size in bytes or with a unit", () => {
    expect(readConfig({ maxPackSize: 1200 }).maxPackSize).toBe(1200);
    expect(readConfig({ maxPackSize: "800 KB" }).maxPackSize).toBe(800_000);
    expect(readConfig({ maxPackSize: "1.5gb" }).maxPackSize).toBe(1_500_000_000);
    expect(readConfig({ maxPackSize: "12B" }).maxPackSize).toBe(12);
  });

  it("refuses trust it can't give, and says why", () => {
    expect(problemsOf({ packs: { "@acme/app": { trust: "trusted" } } })).toEqual([
      'packs["@acme/app"].trust: only a folder in the app may be trusted, never a registry pack; a registry pack that needs more is `permissive`',
    ]);
    expect(problemsOf({ packs: { "@acme/maps": { network: ["https://tiles.example.com"], host: true } } })).toEqual([
      'packs["@acme/maps"].network: only a permissive pack reaches the network',
      'packs["@acme/maps"].host: only a permissive pack is given `host`',
    ]);
    expect(problemsOf({ packs: { "@acme/maps": { trust: "open" } } })).toEqual(['packs["@acme/maps"].trust: `strict`, `permissive` or `trusted`']);
  });

  it("takes origins only, for the network", () => {
    const network = (origin: string) => problemsOf({ packs: { "@acme/maps": { trust: "permissive", network: [origin] } } });
    expect(network("http://tiles.example.com")).toEqual(['packs["@acme/maps"].network: `http://tiles.example.com` must be `https:` (`http:` only for localhost)']);
    expect(network("https://*.example.com")).toEqual(['packs["@acme/maps"].network: `https://*.example.com` has a wildcard: name each origin']);
    expect(network("https://tiles.example.com/v2")).toEqual(['packs["@acme/maps"].network: `https://tiles.example.com/v2` must be an origin alone, without a path']);
    expect(network("tiles")).toEqual(['packs["@acme/maps"].network: `tiles` isn\'t a URL']);
    expect(problemsOf({ packs: { "@acme/maps": { trust: "permissive", network: "https://a.org" } } })).toEqual(['packs["@acme/maps"].network: a list of origins']);
  });

  it("lists every problem at once", () => {
    expect(
      problemsOf({
        pack: {},
        packs: { "acme-maps": { trsut: "permissive", ambient: "yes", version: "latest!" }, "@a/b": "strict" },
        imports: "some",
        deny: [{ pack: "bad", version: "~~", reason: "x" }, { pack: "@a/c" }],
        maxPackSize: "large",
        libraries: ["D3!", { name: "d3" }],
      }),
    ).toEqual([
      "pack: not a key of the configuration; it takes $schema, packs, imports, deny, maxPackSize, libraries, registries, fetch",
      'packs["acme-maps"]: a pack is a scoped name (`@scope/pack`) or a folder path (`./components`)',
      'packs["acme-maps"].trsut: not an option; a pack takes trust, network, host, ambient, version',
      'packs["acme-maps"].ambient: true or false',
      'packs["acme-maps"].version: a semver range, such as `^1.2.0`',
      'packs["@a/b"]: its options are an object, `{}` for none',
      "imports: `any` or `listed`",
      "deny[0].pack: a scoped name, `@scope/pack`",
      "deny[0].version: a semver range, such as `<2`",
      "deny[1]: `{pack, reason}`, and `version` for some versions only",
      'maxPackSize: bytes, or a size such as `"5 MB"`',
      'libraries[0]: an npm package name, such as `"d3"`, or `{ "name": "d3", "imports": ["d3"] }`',
      'libraries[1]: an npm package name, such as `"d3"`, or `{ "name": "d3", "imports": ["d3"] }`',
    ]);
  });

  it("refuses what isn't the shape it takes", () => {
    expect(problemsOf([])).toEqual(["the configuration is an object"]);
    expect(problemsOf({ packs: [], deny: {}, libraries: "d3", maxPackSize: 0 })).toEqual([
      "packs: an object, by pack name or folder path",
      "deny: a list of `{pack, version?, reason}`",
      'maxPackSize: bytes, or a size such as `"5 MB"`',
      "libraries: a list of npm package names, or `{name, imports}`",
    ]);
    expect(problemsOf({ packs: { "@acme/maps": { trust: "permissive", host: "yes" } } })).toEqual(['packs["@acme/maps"].host: true or false']);
  });

  it("names where the configuration came from", () => {
    expect(() => readConfig({ imports: "some" }, "package.json (compmd)")).toThrow("package.json (compmd):\n- imports: `any` or `listed`");
  });
});

describe("policyOf", () => {
  it("gives the processor's policy and ambient packs, folder packs by their names", () => {
    const config = readConfig({
      packs: {
        "@kit/mermaid": {},
        "@acme/maps": { trust: "permissive", network: ["https://tiles.example.com"] },
        "./components": { trust: "trusted", ambient: true },
      },
      deny: [{ pack: "@bad/pack", reason: "misuse" }],
    });
    const { policy, ambient } = policyOf(config, (key) => (isFolder(key) ? "@acme/app" : key), { "@huge/pack": "larger than 5 MB" });
    expect(policy).toEqual({
      packs: {
        "@kit/mermaid": { level: "strict" },
        "@acme/maps": { level: "permissive", network: ["https://tiles.example.com"], host: false },
        "@acme/app": { level: "trusted" },
      },
      imports: "any",
      deny: [{ pack: "@bad/pack", reason: "misuse" }],
      refused: { "@huge/pack": "larger than 5 MB" },
    });
    expect(ambient).toEqual(["@acme/app"]);
    expect(policyOf(readConfig({}), (key) => key).policy).toEqual({ packs: {}, imports: "any", deny: [] });
  });
});

describe("isFolder", () => {
  it("tells a folder path from a pack name", () => {
    expect(isFolder("./components")).toBe(true);
    expect(isFolder("../shared/kit")).toBe(true);
    expect(isFolder("@acme/app")).toBe(false);
  });
});
