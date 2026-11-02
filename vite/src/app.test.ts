import { createHash } from "node:crypto";
import { mkdtemp, mkdir, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppError, configOf, loadApp, trustedComponents } from "./app.ts";
import { ConfigError } from "./config.ts";
import { readPack } from "./packs.ts";

let root: string;

/** Writes `files` under the app's root, by `/`-separated path. */
async function write(files: Record<string, string | object>): Promise<void> {
  for (const [path, content] of Object.entries(files)) {
    const file = join(root, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, typeof content === "string" ? content : JSON.stringify(content));
  }
}

const LINE_MODULE = "export default function mount(target) { target.textContent = 'line'; }\n";

/** An app with a trusted folder pack, a strict pack with a built module and a source component, and a permissive one. */
const APP = {
  "package.json": { name: "app", dependencies: { "@acme/charts": "2.1.0", "@acme/maps": "3.0.0" } },
  "compmd.config.json": {
    packs: {
      "./components": { trust: "trusted", ambient: true },
      "@acme/maps": { trust: "permissive", network: ["https://tiles.example.com"], version: "^3" },
      "@acme/charts": {},
    },
  },
  "components/package.json": { name: "@acme/app", version: "1.0.0" },
  "components/index.comp.md": `---
md:
  import:
    - from: ./Stars.tsx
      names: [{name: Stars, as: stars}]
    - from: ./rating.svelte
      names: "*"
  export:
    pack: "@acme/app"
    names: [stars, {name: rating, as: rating-summary}]
---

\`\`\`compmd component=stars signature
params:
  value: number
\`\`\`

\`\`\`compmd component=rating signature
params:
  product: string
\`\`\`
`,
  "components/Stars.tsx": "export default function Stars() { return null; }",
  "components/rating.svelte": "<p>rating</p>",
  "node_modules/@acme/charts/package.json": { name: "@acme/charts", version: "2.1.0" },
  "node_modules/@acme/charts/index.comp.md": `---
md:
  import:
    - from: ./dist/line.js
      names: [line]
    - from: ./bar.jsx
      names: [bar]
  export:
    pack: "@acme/charts"
    version: "2.1.0"
    names: [line, bar]
---

\`\`\`compmd component=line signature
params:
  rows: array
\`\`\`

\`\`\`compmd component=bar signature
params:
  rows: array
\`\`\`
`,
  "node_modules/@acme/charts/dist/line.js": LINE_MODULE,
  "node_modules/@acme/charts/bar.jsx": "export default () => null;",
  "node_modules/@acme/maps/package.json": { name: "@acme/maps", version: "3.0.0" },
  "node_modules/@acme/maps/index.comp.md": `---
md:
  export:
    pack: "@acme/maps"
    names: [map]
---

\`\`\`jsx component=map runtime=react@19
export default () => null;
\`\`\`

\`\`\`compmd component=map signature
params:
  place: string
\`\`\`
`,
  "pnpm-lock.yaml": "lockfileVersion: '9.0'\npackages:\n  '@acme/charts@2.1.0':\n    resolution: {integrity: sha512-charts}\n",
};

beforeEach(async () => {
  root = join(await mkdtemp(join(tmpdir(), "compmd-vite-")), "app");
  await write(APP);
});

afterEach(async () => {
  await rm(dirname(root), { recursive: true, force: true });
});

describe("configOf", () => {
  it("reads compmd.config.json, or package.json's compmd key, or none", async () => {
    expect((await configOf(root)).packs.size).toBe(3);
    await rm(join(root, "compmd.config.json"));
    expect((await configOf(root)).packs.size).toBe(0);
    await write({ "package.json": { name: "app", compmd: { imports: "listed" } } });
    expect((await configOf(root)).imports).toBe("listed");
  });

  it("takes a pack's `compmd`, its document's path, for no configuration", async () => {
    await rm(join(root, "compmd.config.json"));
    await write({ "package.json": { name: "@kit/badge", compmd: "./index.comp.md" } });
    expect((await configOf(root)).packs.size).toBe(0);
    await write({ "compmd.config.json": { imports: "listed" } });
    expect((await configOf(root)).imports).toBe("listed");
  });

  it("takes the configuration the plugin's options give over the files, and checks it the same way", async () => {
    expect((await configOf(root, { imports: "listed" })).imports).toBe("listed");
    expect((await loadApp(root, { packs: { "./components": { trust: "trusted" } } })).trusted.map(({ name }) => name)).toEqual(["stars", "rating-summary"]);
    await expect(configOf(root, { imports: "some" })).rejects.toThrow(/the plugin's `config`/);
  });

  it("refuses both, JSON that isn't, and a wrong configuration", async () => {
    await write({ "package.json": { name: "app", compmd: {} } });
    await expect(configOf(root)).rejects.toThrow("the configuration is in both compmd.config.json and package.json's `compmd` key: keep one");
    await write({ "compmd.config.json": "{ packs: " });
    await rm(join(root, "package.json"));
    await expect(configOf(root)).rejects.toThrow(/compmd\.config\.json isn't JSON/);
    await write({ "compmd.config.json": { imports: "some" } });
    await expect(configOf(root)).rejects.toThrow(ConfigError);
  });
});

describe("trustedComponents", () => {
  it("names each component as its pack exports it, with its binding", async () => {
    const pack = await readPack(join(root, "components"));
    expect(trustedComponents(pack)).toEqual([
      { name: "stars", file: join(root, "components", "Stars.tsx"), binding: "react" },
      { name: "rating-summary", file: join(root, "components", "rating.svelte"), binding: "svelte" },
    ]);
  });

  it("takes a wildcard export, and plain components", async () => {
    await write({
      "kit/package.json": { name: "@acme/kit" },
      "kit/index.comp.md": "---\nmd:\n  import:\n    - from: ./gauge.js\n      names: [gauge]\n    - from: \"@other/x@1\"\n      names: \"*\"\n  export:\n    names: \"*\"\n---\n",
      "kit/gauge.js": "export default function mount() {}",
    });
    expect(trustedComponents(await readPack(join(root, "kit")))).toEqual([{ name: "gauge", file: join(root, "kit", "gauge.js"), binding: "plain" }]);
  });
});

describe("loadApp", () => {
  it("gives the processor every pack, with the policy and the ambient packs", async () => {
    const app = await loadApp(root);
    expect(Object.keys(app.request.packs).sort()).toEqual(["@acme/app", "@acme/charts", "@acme/maps"]);
    expect(app.request.ambient).toEqual(["@acme/app"]);
    expect(app.request.policy.packs).toEqual({
      "@acme/app": { level: "trusted" },
      "@acme/maps": { level: "permissive", network: ["https://tiles.example.com"], host: false },
      "@acme/charts": { level: "strict" },
    });
    const [charts] = app.request.packs["@acme/charts"]!;
    expect(charts!.integrity).toBe("sha512-charts");
    expect(charts!.files["dist/line.js"]).toBe("");
    expect(charts!.files["bar.jsx"]).toBe("export default () => null;");
    expect(app.request.packs["@acme/app"]![0]!.files["Stars.tsx"]).toBe("");
    expect(app.warnings).toEqual([]);
  });

  it("serves each built module, with its file's hash, and mounts the trusted components", async () => {
    const app = await loadApp(root);
    expect(app.modules).toEqual({
      "@acme/charts@2.1.0/dist/line.js": {
        path: "packs/@acme/charts@2.1.0/dist/line.js",
        integrity: `sha384-${createHash("sha384").update(LINE_MODULE).digest("base64")}`,
      },
    });
    // An installed pack is read where its folder really is, behind `node_modules`' links.
    expect(app.served.get("packs/@acme/charts@2.1.0/dist/line.js")).toBe(join(await realpath(root), "node_modules", "@acme", "charts", "dist", "line.js"));
    expect(app.trusted.map(({ name, binding }) => `${name}:${binding}`)).toEqual(["stars:react", "rating-summary:svelte"]);
  });

  it("leaves out a pack larger than the limit, and says so", async () => {
    await write({ "compmd.config.json": { packs: { "./components": { trust: "trusted" } }, maxPackSize: "2 KB" }, "node_modules/@acme/charts/dist/line.js": `${LINE_MODULE}//${"x".repeat(2000)}\n` });
    const app = await loadApp(root);
    expect(Object.keys(app.request.packs).sort()).toEqual(["@acme/app", "@acme/maps"]);
    expect(app.request.policy.refused?.["@acme/charts"]).toMatch(/^its files are \d+ bytes, more than the 2000 this app serves$/);
    expect(app.modules).toEqual({});
    expect(app.warnings).toEqual([`@acme/charts is left out: ${app.request.policy.refused?.["@acme/charts"] ?? ""}`]);
  });

  it("stops at a listed pack that isn't installed or of the version asked, and a library that isn't installed", async () => {
    await write({ "compmd.config.json": { packs: { "@kit/mermaid": {}, "@acme/maps": { version: "^4" } }, libraries: ["d3"] } });
    await expect(loadApp(root)).rejects.toThrow(
      new AppError(
        [
          "@kit/mermaid is listed in the configuration, but it isn't installed: `npm install @kit/mermaid`",
          "@acme/maps is 3.0.0, which isn't `^4` as the configuration asks",
          "the library d3 isn't installed: `npm install d3`",
        ].join("\n"),
      ),
    );
  });

  it("stops at an error in a listed pack, and reports its warnings", async () => {
    await write({
      "node_modules/@acme/maps/index.comp.md": "---\nmd:\n  export:\n    pack: \"@acme/maps\"\n    names: [map, pin]\n---\n\n```jsx component=map\nexport default () => null;\n```\n\n```compmd component=map signature\nparams:\n  place: string\n```\n\n```jsx component=pin runtime=react@19\nexport default () => null;\n```\n",
    });
    await expect(loadApp(root)).rejects.toThrow(/the listed packs have errors:\n- @acme\/maps@3\.0\.0\/index\.comp\.md:\d+:\d+: F029 /);
    await write({
      "node_modules/@acme/maps/index.comp.md": "---\nmd:\n  export:\n    pack: \"@acme/maps\"\n    names: [map]\n---\n\n```jsx component=map\nexport default () => null;\n```\n\n```compmd component=map signature\nparams:\n  place: string\n```\n",
    });
    const app = await loadApp(root);
    expect(app.warnings).toHaveLength(1);
    expect(app.warnings[0]).toMatch(/^@acme\/maps@3\.0\.0\/index\.comp\.md:\d+:\d+: W117 /);
  });
});
