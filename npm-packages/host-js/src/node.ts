// For Node: the WebAssembly `respond` decides with, loaded from its file as this module is imported (Node's `fetch`
// reads no `file:` URL), and a bundle in a folder, what `@compmd/vite` and `buildBundle` write (`manifest.json`,
// `frames/<hash>/`, `packs/<release>/`).

import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { initSync, manifest as readManifest } from "../pkg/compmd_host_wasm.js";
import type { Bundle, Place } from "./respond.ts";

initSync({ module: readFileSync(new URL("../pkg/compmd_host_wasm_bg.wasm", import.meta.url)) });

export { respond, type Bundle, type Place, type Store } from "./respond.ts";

/** What `folderBundle` reads of a bundle's manifest, as compmd-host reads it (`manifest` in `src/lib.rs`). */
interface Manifest {
  hash: string;
  packs: { pack: string; version: string }[];
}

/**
 * The bundle in `folder`, read from its `manifest.json` as compmd-host reads it (its format, the frames' hash and every
 * name checked): its frames' files, and the files of the packs its manifest lists. A file is read when it is asked for.
 *
 * @throws Error when the folder has no manifest, or one compmd-host can't use, saying why.
 *
 * @example
 * const bundle = await folderBundle("dist/compmd");
 * createServer(async (incoming, outgoing) => { … await respond(request, bundle) … });
 */
export async function folderBundle(folder: string): Promise<Bundle> {
  const read = JSON.parse(readManifest(await readFile(join(folder, "manifest.json"), "utf8"))) as { manifest: Manifest } | { error: string };
  if ("error" in read) throw new Error(`the bundle in ${folder} can't be served: ${read.error}`);
  const { manifest } = read;
  const releases = new Set(manifest.packs.map(({ pack, version }) => `${pack}@${version}`));
  const readFrom = async (path: string): Promise<Uint8Array | undefined> => {
    try {
      return new Uint8Array(await readFile(join(folder, ...path.split("/"))));
    } catch {
      return undefined;
    }
  };
  return {
    hash: manifest.hash,
    frameFile: (path) => readFrom(`frames/${manifest.hash}/${path}`),
    packFile: ({ pack, version, path }: Place) => (releases.has(`${pack}@${version}`) ? readFrom(`packs/${pack}@${version}/${path}`) : Promise.resolve(undefined)),
  };
}
