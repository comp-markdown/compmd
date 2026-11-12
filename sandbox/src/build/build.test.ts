// `@compmd/sandbox/build`, as a host's build uses it: the frame documents and the libraries its frames share, each in
// a folder of its own. `buildFrames` reads the package's built frame files (`from`), here a stand-in for them.

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { builtFrames, folder } from "./frames.testing.ts";
import { buildFrames, buildLibraries, libraryDefinition, readLibraries, RUNTIMES } from "./index.ts";
import { permissivePolicy, strictPolicy } from "./policy.ts";

describe("buildFrames", () => {
  it("writes strict.html and permissive.html, each with its policy first and the import map, next to the runtime", async () => {
    const out = await folder();
    await buildFrames({ out, from: await builtFrames() });
    const map = JSON.stringify({ imports: { react: "./libs/react@19.3.0/index.js" } });
    const strict = await readFile(join(out, "strict.html"), "utf8");
    const permissive = await readFile(join(out, "permissive.html"), "utf8");
    expect(strict).toContain('data-compmd-frame="strict"');
    expect(strict).toContain(`<meta http-equiv="Content-Security-Policy" content="${strictPolicy(map).replaceAll('"', "&quot;")}" />`);
    expect(strict).toContain(`<script type="importmap">${map}</script>`);
    expect(strict.indexOf("Content-Security-Policy"), "the policy before any script").toBeLessThan(strict.indexOf("<script"));
    expect(permissive).toContain(`content="${permissivePolicy(map).replaceAll('"', "&quot;")}"`);
    expect(await readFile(join(out, "runtime.js"), "utf8")).toBe("// the frame runtime\n");
    expect(strict, "no source of its own unless the host names one").not.toContain("data-compmd-source");
  });

  it("names where the host serves the frames in both policies, and on each document for a permissive frame's second", async () => {
    const out = await folder();
    const source = "app://localhost/frames/";
    await buildFrames({ out, from: await builtFrames(), source });
    const map = JSON.stringify({ imports: { react: "./libs/react@19.3.0/index.js" } });
    const strict = await readFile(join(out, "strict.html"), "utf8");
    const permissive = await readFile(join(out, "permissive.html"), "utf8");
    expect(strict).toContain(`content="${strictPolicy(map, source).replaceAll('"', "&quot;")}"`);
    expect(permissive).toContain(`content="${permissivePolicy(map, source).replaceAll('"', "&quot;")}"`);
    for (const document of [strict, permissive]) expect(document).toContain(`data-compmd-source="${source}"`);
  });

  it("names where the host serves the packs beside the frames, so a frame on its scheme loads their modules", async () => {
    const out = await folder();
    const [source, packs] = ["app://localhost/frames/", "app://localhost/packs/"];
    await buildFrames({ out, from: await builtFrames(), source, packs });
    const strict = await readFile(join(out, "strict.html"), "utf8");
    expect(strict).toContain("script-src 'self' app://localhost/frames/ app://localhost/packs/ blob:");
    expect(strict, "the frame's own source alone, for a permissive frame's second policy").toContain(`data-compmd-source="${source}"`);
    await expect(buildFrames({ out, from: await builtFrames(), packs })).rejects.toThrow("needs `source`");
  });

  it("adds the libraries a host offers to the import map, and to the library definition it gives and writes", async () => {
    const out = await folder();
    const d3 = { name: "d3", version: "7.9.0", imports: ["d3"] };
    const definition = await buildFrames({ out, from: await builtFrames(), imports: { d3: "./libs/d3@7.9.0/index.js" }, libraries: [d3] });
    expect(definition).toEqual({ runtimes: [{ name: "react", version: "19.3.0", imports: ["react"] }], libraries: [d3] });
    expect(await readFile(join(out, "strict.html"), "utf8")).toContain('"d3":"./libs/d3@7.9.0/index.js"');
    expect(await libraryDefinition(out), "read back by a host that makes requests later").toEqual(definition);
  });

  it("fails when the frame files it copies aren't there", async () => {
    await expect(buildFrames({ out: await folder(), from: join(await folder(), "missing") })).rejects.toThrow();
  });
});

describe("libraryDefinition", () => {
  it("fails for a folder buildFrames didn't write", async () => {
    await expect(libraryDefinition(await folder())).rejects.toThrow();
  });
});

describe("buildLibraries", () => {
  it("builds each library a host offers into `<name>@<version>/`, gives its import map entries, and writes them with their versions", async () => {
    const root = await folder();
    await writeFile(join(root, "package.json"), JSON.stringify({ name: "host", type: "module" }));
    await mkdir(join(root, "node_modules/tiny"), { recursive: true });
    await writeFile(join(root, "node_modules/tiny/package.json"), JSON.stringify({ name: "tiny", version: "1.2.3", type: "module", exports: "./index.js" }));
    await writeFile(join(root, "node_modules/tiny/index.js"), "export const tiny = () => 'tiny';\n");
    const out = join(root, "public/libs");
    const imports = await buildLibraries([{ name: "tiny", imports: ["tiny"] }], { root, out });
    expect(imports).toEqual({ tiny: "./libs/tiny@1.2.3/index.js" });
    expect(await readFile(join(out, "tiny@1.2.3/index.js"), "utf8")).toContain("tiny");
    expect(await readLibraries(out)).toEqual([{ name: "tiny", version: "1.2.3", imports: ["tiny"] }]);
    expect(JSON.parse(await readFile(join(out, "importmap.json"), "utf8"))).toEqual({ imports });
  });
});

describe("RUNTIMES", () => {
  it("are the runtimes of the spec's bindings (§7.6), by the names component code imports them", () => {
    expect(RUNTIMES.map(({ name }) => name)).toEqual(["react", "react-dom", "svelte"]);
    expect(RUNTIMES.flatMap(({ imports }) => imports)).toEqual(expect.arrayContaining(["react", "react/jsx-runtime", "react-dom/client", "svelte", "svelte/internal/client"]));
  });
});
