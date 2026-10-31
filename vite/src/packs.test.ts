import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { findPacks, integrities, PackError, readPack, releaseOf } from "./packs.ts";

let root: string;

/** Writes `files` under the app's root, by `/`-separated path. */
async function write(files: Record<string, string | object>): Promise<void> {
  for (const [path, content] of Object.entries(files)) {
    const file = join(root, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, typeof content === "string" ? content : JSON.stringify(content));
  }
}

const CHARTS_DOCUMENT = `---
md:
  import:
    - from: ./bar.jsx
      names: [bar]
    - from: ./line.js
      names: [line]
    - from: ./more/more.comp.md
      names: "*"
    - from: "@other/kit@1"
      names: "*"
  export:
    pack: "@acme/charts"
    names: [bar, line]
---
`;

beforeEach(async () => {
  // The app one folder down, so a lock file above it stays in the test's own folder.
  root = join(await mkdtemp(join(tmpdir(), "compmd-vite-")), "app");
  await write({
    "package.json": { name: "app", dependencies: { "@acme/charts": "2.1.0", "@acme/plain": "1.0.0", lodash: "4" }, devDependencies: { "@kit/mermaid": "1.0.0" } },
    "node_modules/@acme/charts/package.json": { name: "@acme/charts", version: "2.1.0", compmd: "./kit.comp.md" },
    "node_modules/@acme/charts/kit.comp.md": CHARTS_DOCUMENT,
    "node_modules/@acme/charts/bar.jsx": "export default () => null;",
    "node_modules/@acme/charts/line.js": "export default function mount() {}",
    "node_modules/@acme/charts/more/more.comp.md": "---\nmd:\n  data:\n    - from: ./colors.json\n      as: colors\n---\n",
    "node_modules/@acme/charts/more/colors.json": "[]",
    "node_modules/@acme/plain/package.json": { name: "@acme/plain", version: "1.0.0" },
    "node_modules/@kit/mermaid/package.json": { name: "@kit/mermaid", version: "1.0.0" },
    "node_modules/@kit/mermaid/index.comp.md": "# A pack\n",
    "node_modules/lodash/package.json": { name: "lodash", version: "4.17.21" },
    "node_modules/lodash/index.comp.md": "",
    "components/package.json": { name: "@acme/app" },
    "components/index.comp.md": "---\nmd:\n  import:\n    - from: ./Stars.tsx\n      names: [{name: Stars, as: stars}]\n---\n",
    "components/Stars.tsx": "export default function Stars() { return null; }",
  });
});

afterEach(async () => {
  await rm(dirname(root), { recursive: true, force: true });
});

describe("findPacks", () => {
  it("finds every scoped dependency that is a pack, and the folder packs", async () => {
    const packs = await findPacks(root, ["./components"]);
    expect(packs.map(({ name, version, folder, document }) => ({ name, version, folder, document }))).toEqual([
      { name: "@acme/charts", version: "2.1.0", folder: undefined, document: "kit.comp.md" },
      { name: "@kit/mermaid", version: "1.0.0", folder: undefined, document: "index.comp.md" },
      { name: "@acme/app", version: "0.0.0", folder: "./components", document: "index.comp.md" },
    ]);
  });

  it("reads every file a pack's documents import, and its size", async () => {
    const [charts] = await findPacks(root, []);
    expect(Object.keys(charts!.files).sort()).toEqual(["bar.jsx", "kit.comp.md", "line.js", "more/colors.json", "more/more.comp.md", "package.json"]);
    expect(charts!.components).toEqual([
      { path: "bar.jsx", language: "jsx" },
      { path: "line.js", language: "js" },
    ]);
    expect(charts!.size).toBe(Object.values(charts!.files).reduce((sum, text) => sum + Buffer.byteLength(text), 0));
  });

  it("refuses a listed folder that isn't one, and a pack that isn't whole", async () => {
    await expect(findPacks(root, ["./missing"])).rejects.toThrow("./missing is listed in the configuration, but it isn't a folder of the app");
    await write({ "components/package.json": { version: "1.0.0" } });
    await expect(findPacks(root, ["./components"])).rejects.toThrow(PackError);
    await write({ "broken/package.json": { name: "@acme/broken" }, "broken/index.comp.md": "---\nmd:\n  import:\n    - from: ./gone.jsx\n      names: [gone]\n---\n" });
    await expect(readPack(join(root, "broken"))).rejects.toThrow("@acme/broken: `gone.jsx` isn't in the pack");
    await write({ "outside/package.json": { name: "@acme/outside" }, "outside/index.comp.md": "---\nmd:\n  import:\n    - from: ../components/Stars.tsx\n      names: [Stars]\n---\n" });
    await expect(readPack(join(root, "outside"))).rejects.toThrow("@acme/outside: `../components/Stars.tsx` leaves the pack");
    await write({ "empty/package.json": { name: "@acme/empty" } });
    await expect(readPack(join(root, "empty"))).rejects.toThrow("@acme/empty: `index.comp.md` isn't in the pack, which needs its document");
  });

  it("leaves a broken frontmatter to the processor", async () => {
    await write({ "odd/package.json": { name: "@acme/odd" }, "odd/index.comp.md": "---\nmd: [unclosed\n---\n" });
    expect(Object.keys((await readPack(join(root, "odd"))).files)).toEqual(["package.json", "index.comp.md"]);
  });
});

describe("integrities", () => {
  it("reads pnpm's lock file, the nearest above the app", async () => {
    await write({
      "../pnpm-lock.yaml": [
        "---",
        "lockfileVersion: '9.0'",
        "settings:",
        "  autoInstallPeers: true",
        "",
        "---",
        "lockfileVersion: '9.0'",
        "packages:",
        "  '@acme/charts@2.1.0':",
        "    resolution: {integrity: sha512-charts}",
        "  /@kit/mermaid@1.0.0:",
        "    resolution: {integrity: sha512-mermaid}",
        "  '@acme/react-kit@1.0.0(react@19.3.0)':",
        "    resolution: {integrity: sha512-react-kit}",
        "  '@acme/local@1.0.0':",
        "    resolution: {directory: ../local}",
        "",
      ].join("\n"),
    });
    const found = await integrities(root);
    expect(found.get("@acme/charts@2.1.0")).toBe("sha512-charts");
    expect(found.get("@kit/mermaid@1.0.0")).toBe("sha512-mermaid");
    expect(found.get("@acme/react-kit@1.0.0")).toBe("sha512-react-kit");
    expect(found.has("@acme/local@1.0.0")).toBe(false);
  });

  it("reads npm's lock file", async () => {
    await write({
      "package-lock.json": {
        lockfileVersion: 3,
        packages: {
          "": { name: "app" },
          "node_modules/@acme/charts": { version: "2.1.0", integrity: "sha512-charts" },
          "node_modules/@acme/charts/node_modules/@kit/mermaid": { version: "0.9.0", integrity: "sha512-old" },
          "packages/local": { name: "@acme/local", version: "1.0.0" },
        },
      },
    });
    const found = await integrities(root);
    expect(found.get("@acme/charts@2.1.0")).toBe("sha512-charts");
    expect(found.get("@kit/mermaid@0.9.0")).toBe("sha512-old");
    expect(found.size).toBe(2);
  });
});

describe("releaseOf", () => {
  it("gives the processor the pack's files, with built modules and trusted components empty", async () => {
    const [charts, , app] = await findPacks(root, ["./components"]);
    const release = releaseOf({ ...charts!, integrity: "sha512-charts" }, { trusted: false });
    expect(release.version).toBe("2.1.0");
    expect(release.integrity).toBe("sha512-charts");
    expect(release.files["bar.jsx"]).toBe("export default () => null;");
    expect(release.files["line.js"]).toBe("");
    expect(releaseOf(app!, { trusted: true }).files).toEqual({ ...app!.files, "Stars.tsx": "" });
    expect(releaseOf(app!, { trusted: true }).integrity).toBeNull();
  });
});
