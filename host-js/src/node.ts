// For Node: the WebAssembly `respond` decides with, loaded from its file as this module is imported (Node's `fetch`
// reads no `file:` URL), and a bundle in a folder, what `@compmd/vite` and `buildBundle` write (`manifest.json`,
// `frames/<hash>/`, `packs/<release>/`).

import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { initSync } from "../pkg/compmd_host_wasm.js";
import type { Bundle, Place } from "./respond.ts";

initSync({ module: readFileSync(new URL("../pkg/compmd_host_wasm_bg.wasm", import.meta.url)) });

export { respond, type Bundle, type Place, type Store } from "./respond.ts";

/** What `folderBundle` reads of a bundle's manifest. */
interface Manifest {
  frames: { hash: string };
  packs?: { pack: string; version: string }[];
}

/**
 * The bundle in `folder`, read from its `manifest.json`: its frames' files, and the files of the packs its manifest
 * lists. A file is read when it is asked for.
 *
 * @throws Error when the folder has no manifest, or one without the frames' hash.
 *
 * @example
 * const bundle = await folderBundle("dist/compmd");
 * createServer(async (incoming, outgoing) => { … await respond(request, bundle) … });
 */
export async function folderBundle(folder: string): Promise<Bundle> {
  const manifest = JSON.parse(await readFile(join(folder, "manifest.json"), "utf8")) as Manifest;
  if (typeof manifest.frames.hash !== "string") throw new Error(`the bundle in ${folder} has a manifest without the frames' hash`);
  const releases = new Set((manifest.packs ?? []).map(({ pack, version }) => `${pack}@${version}`));
  const read = async (path: string): Promise<Uint8Array | undefined> => {
    try {
      return new Uint8Array(await readFile(join(folder, ...path.split("/"))));
    } catch {
      return undefined;
    }
  };
  return {
    hash: manifest.frames.hash,
    frameFile: (path) => read(`frames/${manifest.frames.hash}/${path}`),
    packFile: ({ release, path }: Place) => (releases.has(release) ? read(`packs/${release}/${path}`) : Promise.resolve(undefined)),
  };
}
