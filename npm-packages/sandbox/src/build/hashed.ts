// The frames at content-hashed paths (docs/tauri-plugin.md, decision 2): a webview may cache what a scheme of the
// app's own serves, so a frame document replaced at the same path can keep running its old version after an upgrade.
// Each build goes into a folder named by a hash of its files, with `frames.json` naming it beside them.

import { createHash } from "node:crypto";
import { mkdir, readdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { join, relative, sep } from "node:path";
import { buildFrames, type FramesOptions } from "./frames.ts";
import type { LibraryDefinition } from "./libraries.ts";

/** How many hex digits of the SHA-256 name a build. */
const DIGITS = 16;

/** A build's folder name: what `hashOf` gives. */
const BUILD = /^[0-9a-f]{16}$/;

/** The file beside the builds that names the current one. */
export const FRAMES_JSON = "frames.json";

/**
 * The first 16 hex digits of a SHA-256 over every file under `folder`: each one's path from it, with `/` between its
 * parts and its length in front, then its bytes, in the order of their paths. The same files give the same hash on
 * every system.
 */
export async function hashOf(folder: string): Promise<string> {
  const entries = await readdir(folder, { recursive: true, withFileTypes: true });
  const files = entries
    .filter((entry) => entry.isFile())
    .map((entry) => relative(folder, join(entry.parentPath, entry.name)).split(sep).join("/"))
    .sort();
  const hash = createHash("sha256");
  for (const file of files) {
    // The path's length first, so a path and the bytes after it can't be read as another split.
    hash.update(`${String(file.length)}:${file}`);
    hash.update(await readFile(join(folder, file)));
  }
  return hash.digest("hex").slice(0, DIGITS);
}

/** What `buildHashedFrames` wrote: the build's hash, its folder under `out`, and the frames' library definition. */
export interface HashedFrames {
  hash: string;
  folder: string;
  definition: LibraryDefinition;
}

/**
 * Builds the frames (`buildFrames`, with these `options`) into `options.out/<hash>/`, names that folder in
 * `options.out/frames.json`, `{ "hash": "<hash>" }`, and removes older builds: the other folders whose names are
 * hashes. Nothing else in `out` is touched. The host serves `<hash>/` and reads `frames.json` when it starts.
 *
 * @example
 * const { hash } = await buildHashedFrames({ out: "src-tauri/compmd", source: "compmd://localhost/frames/" });
 * // src-tauri/compmd/frames.json: { "hash": "3f9a0c…" }, and the frames in src-tauri/compmd/3f9a0c…/
 */
export async function buildHashedFrames(options: FramesOptions): Promise<HashedFrames> {
  const staging = join(options.out, ".staging");
  await rm(staging, { recursive: true, force: true });
  await mkdir(staging, { recursive: true });
  const definition = await buildFrames({ ...options, out: staging });
  const hash = await hashOf(staging);
  const folder = join(options.out, hash);
  await rm(folder, { recursive: true, force: true });
  await rename(staging, folder);
  await writeFile(join(options.out, FRAMES_JSON), `${JSON.stringify({ hash })}\n`);
  for (const entry of await readdir(options.out, { withFileTypes: true })) {
    if (entry.isDirectory() && BUILD.test(entry.name) && entry.name !== hash) await rm(join(options.out, entry.name), { recursive: true, force: true });
  }
  return { hash, folder, definition };
}
