// Builds the package into `dist/` with rolldown: the tools' Svelte compiled in, the other `@compmd` packages left to the
// host's bundler, and the workers as files of their own next to the page's code. Not with Vite: its library build turns
// `new Worker(new URL("./worker.js", import.meta.url))` into a URL it marks to be ignored, which a host's bundler would
// then never build, and the workers' imports of the processor would reach the browser as they are.

import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { build, type Plugin } from "rolldown";
import { compile } from "svelte/compiler";

const ROOT = join(import.meta.dirname, "..");

/** `.svelte` files compiled for the browser, their styles injected where they mount: in the tools' shadow root. */
const svelte: Plugin = {
  name: "svelte",
  transform: {
    filter: { id: /\.svelte$/ },
    handler(source, id) {
      const { js, warnings } = compile(source, { filename: id, generate: "client", css: "injected" });
      for (const warning of warnings) this.warn(warning.message);
      return { code: js.code, map: js.map, moduleType: "js" };
    },
  },
};

/** A stylesheet imported with `?inline`, as its text. */
const inlineCss: Plugin = {
  name: "inline-css",
  resolveId: {
    filter: { id: /\.css\?inline$/ },
    async handler(source, importer) {
      const resolved = await this.resolve(source.replace(/\?inline$/, ""), importer);
      return resolved && `${resolved.id}?inline`;
    },
  },
  load: {
    filter: { id: /\.css\?inline$/ },
    async handler(id) {
      const css = await readFile(id.replace(/\?inline$/, ""), "utf8");
      return { code: `export default ${JSON.stringify(css)};`, moduleType: "js" };
    },
  },
};

await build({
  cwd: ROOT,
  input: {
    index: "src/index.ts",
    testing: "src/testing.ts",
    worker: "src/worker.ts",
    "views-worker": "src/views-worker.ts",
  },
  platform: "browser",
  resolve: { conditionNames: ["browser", "import", "default"] },
  external: [/^@compmd\//],
  plugins: [svelte, inlineCss],
  output: {
    dir: "dist",
    format: "es",
    // Every file at the top of `dist/`, so the workers' URLs, relative to the code that makes them, find them.
    entryFileNames: "[name].js",
    chunkFileNames: "[name]-[hash].js",
    cleanDir: true,
  },
});
console.log("@compmd/devtools in dist/");
