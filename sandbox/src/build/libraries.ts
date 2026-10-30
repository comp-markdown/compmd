// The libraries every frame shares through its import map: compiled components and built modules import them as their
// authors write them (`react`, `svelte`), and the map gives them the one copy the frames have. By default the
// runtimes of the spec's bindings; a host adds the libraries it offers (docs/sandbox.md, Q4).
//
// One build per npm package. Its import names are entry points, split into shared chunks, so modules that share state
// (`svelte` and `svelte/internal/client`) share one copy. The other packages stay external, so `react-dom` uses the
// one React. A CommonJS package gets entries that list its named exports.

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { join } from "node:path";
import type { Plugin } from "vite";

export interface Library {
  /** The npm package. */
  name: string;
  /** The import names components use. */
  imports: string[];
  /** Whether it is CommonJS, so its entries list its named exports. */
  commonjs?: boolean;
}

/** The runtimes of the spec's bindings (§7.6): React 19 and React DOM, Svelte 5. */
export const RUNTIMES: Library[] = [
  { name: "react", imports: ["react", "react/jsx-runtime"], commonjs: true },
  { name: "react-dom", imports: ["react-dom", "react-dom/client"], commonjs: true },
  {
    name: "svelte",
    imports: [
      "svelte",
      "svelte/internal/client",
      "svelte/internal/disclose-version",
      "svelte/internal/flags/legacy",
      "svelte/internal/flags/async",
      "svelte/reactivity",
      "svelte/store",
      "svelte/transition",
      "svelte/easing",
      "svelte/motion",
      "svelte/animate",
      "svelte/events",
      "svelte/attachments",
    ],
  },
];

/** Every import name the runtimes' map resolves, which the frame runtime leaves for the map. */
export const SHARED = new Set(RUNTIMES.flatMap((library) => library.imports));

/** The npm package of an import name. */
export const packageOf = (specifier: string): string => (specifier.startsWith("@") ? specifier.split("/").slice(0, 2).join("/") : (specifier.split("/")[0] ?? specifier));

const ENTRY = "\0compmd-entry:";
const SHIM = "\0compmd-shim:";

/** The output name of an import name: `react-dom/client` is `client`, `react` is `index`. */
const entryName = (library: Library, specifier: string): string => specifier.slice(library.name.length + 1).replaceAll("/", "-") || "index";

/**
 * The entries as virtual modules. A `require()` of another library, as `react-dom` has of React, can't stay a call in
 * an ES module: it goes through a module that imports the library, which the import map resolves.
 */
function entries(library: Library, all: readonly Library[], require: NodeJS.Require): Plugin {
  const isOther = (id: string) => packageOf(id) !== library.name && all.some((other) => other.name === packageOf(id));
  const source = (specifier: string) => {
    if (!library.commonjs) return `export * from "${specifier}";\n`;
    const names = Object.keys(require(specifier) as object).filter((name) => name !== "default");
    return `import cjs from "${specifier}";\nexport const { ${names.join(", ")} } = cjs;\nexport default cjs;\n`;
  };
  return {
    name: "compmd-library-entries",
    // Before Vite's own resolving, which would bundle the other libraries.
    enforce: "pre",
    resolveId(id, _importer, options) {
      if (id.startsWith(ENTRY) || id.startsWith(SHIM)) return id;
      if (!isOther(id)) return undefined;
      return (options as { kind?: string }).kind === "require-call" ? `${SHIM}${id}` : { id, external: true };
    },
    load(id) {
      if (id.startsWith(SHIM)) {
        const specifier = id.slice(SHIM.length);
        return `export * from "${specifier}";\nexport { default } from "${specifier}";\n`;
      }
      return id.startsWith(ENTRY) ? source(id.slice(ENTRY.length)) : undefined;
    },
  };
}

export interface LibrariesOptions {
  /** The folder whose `node_modules` holds the libraries. */
  root: string;
  /** Where `<name>@<version>/` goes for each library. */
  out: string;
  /** What the import map's entries start with, where `out` is served next to the frame documents: `./libs/`. */
  prefix?: string;
  /** The libraries the frames share already, which these may import and stay external. */
  shared?: readonly Library[];
}

/**
 * Builds `libraries` into `options.out`, and gives the import map's entries for them.
 *
 * @example
 * const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root: import.meta.dirname, out: "public/libs" });
 */
export async function buildLibraries(libraries: readonly Library[], options: LibrariesOptions): Promise<Record<string, string>> {
  const { build } = await import("vite");
  const require = createRequire(join(options.root, "package.json"));
  const all = [...(options.shared ?? []), ...libraries];
  const prefix = options.prefix ?? "./libs/";
  const imports: Record<string, string> = {};
  await mkdir(options.out, { recursive: true });
  for (const library of libraries) {
    // From the root's own `node_modules`, since a package's `exports` may not give its `package.json` (d3's doesn't).
    const { version } = JSON.parse(await readFile(join(options.root, "node_modules", library.name, "package.json"), "utf8")) as { version: string };
    const dir = `${library.name}@${version}`;
    await build({
      configFile: false,
      root: options.root,
      logLevel: "warn",
      plugins: [entries(library, all, require)],
      define: { "process.env.NODE_ENV": '"production"' },
      build: {
        outDir: join(options.out, dir),
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
    for (const specifier of library.imports) imports[specifier] = `${prefix}${dir}/${entryName(library, specifier)}.js`;
  }
  await writeFile(join(options.out, "importmap.json"), `${JSON.stringify({ imports }, null, 2)}\n`);
  return imports;
}
