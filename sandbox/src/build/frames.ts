// The frame documents a host serves (docs/sandbox.md, Q3): `strict.html` and `permissive.html`, each with its policy
// and the import map, next to the frame runtime and the shared libraries, which the package ships built.

import { cp, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import type { Level } from "../protocol/messages.ts";
import { permissivePolicy, strictPolicy } from "./policy.ts";

export interface FramesOptions {
  /** The folder to write the frame documents and their files to, which the host serves at `base`. */
  out: string;
  /** The full URL `out` is served at, ending in `/`: a permissive frame names `strict.html` by it. */
  base: string;
  /** More import map entries, for libraries the host offers and serves (`buildLibraries`). */
  imports?: Readonly<Record<string, string>>;
  /** The package's built frame files; by default its own `dist/frames/`. */
  from?: string;
}

/** A frame document of `level`: its policy first, then the import map, then the frame runtime. */
function frameDocument(level: Level, policy: string, importMap: string): string {
  const escape = (text: string) => text.replaceAll("&", "&amp;").replaceAll('"', "&quot;");
  return `<!doctype html>
<html lang="en" data-compmd-frame="${level}">
  <head>
    <meta charset="utf-8" />
    <meta http-equiv="Content-Security-Policy" content="${escape(policy)}" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <script type="importmap">${importMap}</script>
    <style>html, body { margin: 0; } #part:empty, #page:empty { display: none; }</style>
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
 * Writes the frame documents for `options.base` into `options.out`, with the frame runtime and the shared libraries.
 *
 * @throws Error when `base` isn't a full URL ending in `/`.
 * @example
 * await buildFrames({ out: "public/compmd", base: "https://app.example/compmd/" });
 */
export async function buildFrames(options: FramesOptions): Promise<void> {
  if (!URL.canParse(options.base) || !options.base.endsWith("/")) throw new Error(`\`${options.base}\` isn't a full URL ending in /`);
  const from = options.from ?? fileURLToPath(new URL("../frames/", import.meta.url));
  await cp(from, options.out, { recursive: true });
  const shared = JSON.parse(await readFile(join(from, "libs/importmap.json"), "utf8")) as { imports: Record<string, string> };
  const importMap = JSON.stringify({ imports: { ...shared.imports, ...options.imports } });
  const strictUrl = new URL("strict.html", options.base).href;
  await writeFile(join(options.out, "strict.html"), frameDocument("strict", strictPolicy(importMap), importMap));
  await writeFile(join(options.out, "permissive.html"), frameDocument("permissive", permissivePolicy(importMap, strictUrl), importMap));
}
