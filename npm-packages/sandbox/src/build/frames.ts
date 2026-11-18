// The frame documents a host serves (docs/running-components-in-the-sandbox.md, Q3): `strict.html` and
// `permissive.html`, each with its policy and the import map, next to the frame runtime and the shared libraries,
// which the package ships built.

import { cp, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import type { Level } from "../protocol/messages.ts";
import { readLibraries, type BuiltLibrary, type LibraryDefinition } from "./libraries.ts";
import { permissivePolicy, strictPolicy } from "./policy.ts";

/** The file `buildFrames` writes the frames' library definition to. */
const DEFINITION = "library-definition.json";

export interface FramesOptions {
  /** The folder to write the frame documents and their files to, which the host serves where it likes. */
  out: string;
  /** More import map entries, for libraries the host offers and serves (`buildLibraries`). */
  imports?: Readonly<Record<string, string>>;
  /** Those libraries with their versions (`readLibraries`), for the library definition. */
  libraries?: readonly BuiltLibrary[];
  /** The package's built frame files; by default its own `dist/frames/`. */
  from?: string;
  /**
   * Where the host serves the frames, when it serves them on a scheme other than http and https, such as a desktop
   * app's own scheme: a scheme, a host and a path, `app://localhost/frames/`. WebKit's `'self'` matches nothing in a
   * sandboxed frame on such a scheme, so the frames' policies name this too wherever they say `'self'`. A path keeps
   * them to the frames' own files on that scheme.
   */
  source?: string;
  /**
   * Where the host serves the packs' files on that scheme, `app://localhost/packs/`, which a frame loads built modules
   * from by their URL (docs/hosting.md, decision 5): named beside `source`. Only with `source`.
   */
  packs?: string;
}

/**
 * A frame document of `level`: its policy first, then the import map, then the frame runtime. Its body holds its
 * children's margins (`flow-root`), so the size it reports is all of its content and its frame needs no scrollbar.
 * The frames' `source`, when there is one, is on the document too, for the policy a permissive frame adds.
 */
function frameDocument(level: Level, policy: string, importMap: string, source: string | undefined): string {
  const escape = (text: string) => text.replaceAll("&", "&amp;").replaceAll('"', "&quot;");
  const own = source === undefined ? "" : ` data-compmd-source="${escape(source)}"`;
  return `<!doctype html>
<html lang="en" data-compmd-frame="${level}"${own}>
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="Content-Security-Policy" content="${escape(policy)}" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <script type="importmap">${importMap}</script>
    <style>html, body { margin: 0; } body { display: flow-root; } #part:empty, #page:empty { display: none; }</style>
    <script type="module" src="./runtime.js"></script>
  </head>
  <body>
    <main id="page"></main>
    <div id="part"></div>
  </body>
</html>
`;
}

/**
 * Writes the frame documents into `options.out`, with the frame runtime and the shared libraries. They work wherever
 * the host serves them: a permissive frame names the `strict.html` next to it in its second policy. It gives the
 * frames' library definition, and writes it there too, for a host that reads it later (`libraryDefinition`).
 *
 * @example
 * const { runtimes, libraries } = await buildFrames({ out: "public/compmd" });
 * // runtimes: [{ name: "react", version: "19.3.0", imports: ["react", "react/jsx-runtime"] }, …]
 */
export async function buildFrames(options: FramesOptions): Promise<LibraryDefinition> {
  const from = options.from ?? fileURLToPath(new URL("../frames/", import.meta.url));
  await cp(from, options.out, { recursive: true });
  const shared = JSON.parse(await readFile(join(from, "libs/importmap.json"), "utf8")) as { imports: Record<string, string> };
  const importMap = JSON.stringify({ imports: { ...shared.imports, ...options.imports } });
  const { source, packs } = options;
  if (packs !== undefined && source === undefined) throw new Error("`packs` names where packs are served on the frames' own scheme, so it needs `source`");
  const sources = [source, packs].filter((named) => named !== undefined);
  await writeFile(join(options.out, "strict.html"), frameDocument("strict", strictPolicy(importMap, sources), importMap, source));
  await writeFile(join(options.out, "permissive.html"), frameDocument("permissive", permissivePolicy(importMap, sources), importMap, source));
  const definition: LibraryDefinition = { runtimes: await readLibraries(join(from, "libs")), libraries: [...(options.libraries ?? [])] };
  await writeFile(join(options.out, DEFINITION), `${JSON.stringify(definition, null, 2)}\n`);
  return definition;
}

/**
 * The library definition of the frames `buildFrames` wrote into `out`, read back.
 *
 * @example
 * const { runtimes, libraries } = await libraryDefinition("public/compmd");
 */
export async function libraryDefinition(out: string): Promise<LibraryDefinition> {
  return JSON.parse(await readFile(join(out, DEFINITION), "utf8")) as LibraryDefinition;
}
