// Builds the libraries that compiled components and the editor's own code share into `public/libs/<name>@<version>/`,
// with `public/libs/importmap.json`, the import map both pages load. Components import `react` or `svelte` as their
// authors write them, and the import map gives them the one copy the page has.
//
// One build per npm package. Its import names are entry points, split into shared chunks, so modules that share
// state (`svelte` and `svelte/internal/client`) share one copy. The other packages stay external, so `react-dom`
// uses the one React. A CommonJS package gets entries that list its named exports.

import { mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { join } from "node:path";
import { build, type Plugin } from "vite";
import { LIBRARIES, type Library, packageOf } from "../libraries.ts";

const ROOT = join(import.meta.dirname, "..");
const OUT = join(ROOT, "public/libs");
const require = createRequire(join(ROOT, "package.json"));
const ENTRY = "\0compmd-entry:";
const SHIM = "\0compmd-shim:";

/** The output name of an import name: `react-dom/client` is `client`, `react` is `index`. */
const entryName = (library: Library, specifier: string): string => specifier.slice(library.name.length + 1).replaceAll("/", "-") || "index";

/** An entry's source: the import name's exports, listed for a CommonJS package. */
function entrySource(library: Library, specifier: string): string {
  if (!library.commonjs) return `export * from "${specifier}";\n`;
  const names = Object.keys(require(specifier) as object).filter((name) => name !== "default");
  return `import cjs from "${specifier}";\nexport const { ${names.join(", ")} } = cjs;\nexport default cjs;\n`;
}

/** Whether `id` is another library's, which the import map resolves. */
const isOther = (library: Library, id: string): boolean =>
  packageOf(id) !== library.name && LIBRARIES.some((other) => other.name === packageOf(id));

/**
 * The entries as virtual modules. A `require()` of another library, as `react-dom` has of React, can't stay a call in
 * an ES module: it goes through a module that imports the library, which the import map resolves.
 */
function entries(library: Library): Plugin {
  return {
    name: "compmd-library-entries",
    // Before Vite's own resolving, which would bundle the other libraries.
    enforce: "pre",
    resolveId(id, _importer, options) {
      if (id.startsWith(ENTRY) || id.startsWith(SHIM)) return id;
      if (!isOther(library, id)) return undefined;
      return (options as { kind?: string }).kind === "require-call" ? `${SHIM}${id}` : { id, external: true };
    },
    load(id) {
      if (id.startsWith(SHIM)) {
        const specifier = id.slice(SHIM.length);
        return `export * from "${specifier}";\nexport { default } from "${specifier}";\n`;
      }
      return id.startsWith(ENTRY) ? entrySource(library, id.slice(ENTRY.length)) : undefined;
    },
  };
}

async function buildLibrary(library: Library): Promise<Record<string, string>> {
  const { version } = JSON.parse(await readFile(require.resolve(`${library.name}/package.json`), "utf8")) as { version: string };
  const dir = `${library.name}@${version}`;
  await build({
    configFile: false,
    root: ROOT,
    logLevel: "warn",
    plugins: [entries(library)],
    define: { "process.env.NODE_ENV": '"production"' },
    build: {
      outDir: join(OUT, dir),
      emptyOutDir: true,
      copyPublicDir: false,
      minify: true,
      rolldownOptions: {
        input: Object.fromEntries(library.imports.map((specifier) => [entryName(library, specifier), `${ENTRY}${specifier}`])),
        preserveEntrySignatures: "strict",
        output: { format: "es", entryFileNames: "[name].js", chunkFileNames: "chunks/[hash].js" },
      },
    },
  });
  return Object.fromEntries(library.imports.map((specifier) => [specifier, `./libs/${dir}/${entryName(library, specifier)}.js`]));
}

await rm(OUT, { recursive: true, force: true });
await mkdir(OUT, { recursive: true });
const imports: Record<string, string> = {};
for (const library of LIBRARIES) Object.assign(imports, await buildLibrary(library));
await writeFile(join(OUT, "importmap.json"), `${JSON.stringify({ imports }, null, 2)}\n`);
console.log(`Libraries: ${LIBRARIES.map((library) => library.name).join(", ")}`);
