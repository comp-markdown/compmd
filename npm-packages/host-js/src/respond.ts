// The serving contract (docs/hosting.md, decision 8) over the Fetch API: a bundle's frames and the packs the bundle or
// the store holds, with the headers that sandbox them, and a refusal, saying why, for anything else. What a request
// asks for, and the headers, are compmd-host's decision, as WebAssembly (`src/lib.rs`), so both answer alike; this
// reads the file and makes the Response. Both run the contract's cases (`crates/host/fixtures/respond/`).

import init, { notServed, route } from "../pkg/compmd_host_wasm.js";

/** A pack file's place, as the contract's paths name it: its release, `@kit/mermaid@1.2.0`, and its path in it. */
export interface Place {
  release: string;
  path: string;
}

/** What `respond` serves from: a bundle's frames and the packs it holds (`folderBundle` reads one from a folder). */
export interface Bundle {
  /** The frames' hash, from the bundle's manifest. */
  readonly hash: string;
  /** The bytes of the frames' file at `path`, or `undefined` when it has none. */
  frameFile(path: string): Promise<Uint8Array | undefined>;
  /** The bytes of a file of a pack the bundle holds (its manifest lists), or `undefined` for any other. */
  packFile(place: Place): Promise<Uint8Array | undefined>;
}

/** Where packs the bundle doesn't hold are kept, such as those a host fetched from a registry. */
export interface Store {
  /** The bytes of a file of a release the store holds, or `undefined` for any other. */
  file(place: Place): Promise<Uint8Array | undefined>;
}

/** What compmd-host decides for a request (`route` in `src/lib.rs`). */
type Answer =
  | { kind: "frame"; path: string; headers: Record<string, string> }
  | { kind: "pack"; release: string; path: string; headers: Record<string, string> }
  | { kind: "refused"; status: number; why: string; headers: Record<string, string> };

/** The module, loaded once: from its file next to this one, unless the host loaded it already (`init`, `initSync`). */
let loading: Promise<unknown> | undefined;

/**
 * The answer to `request`: the frames' file or pack file it names, from `bundle` or, for a pack the bundle doesn't
 * hold, from `store`; a `405` for another method than `GET`; a `404` for anything else. Each refusal says why, in
 * plain text, readable from any origin.
 *
 * The paths it answers are `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<release>/<path>`, where a
 * path can't leave its folder. A host mounted under a prefix strips it first. In Node, import `@compmd/host/node`,
 * which loads the WebAssembly from its file; a worker that bundles it gives it with `initSync({ module })` first.
 *
 * @example
 * // a worker
 * export default { fetch: (request) => respond(request, bundle, store) };
 */
export async function respond(request: Request, bundle: Bundle, store?: Store): Promise<Response> {
  await (loading ??= init());
  const answer = JSON.parse(route(request.method, new URL(request.url).pathname, bundle.hash)) as Answer;
  if (answer.kind === "refused") return refused(answer);
  const body = answer.kind === "frame" ? await bundle.frameFile(answer.path) : await packFile(bundle, store, { release: answer.release, path: answer.path });
  return body ? new Response(body as Uint8Array<ArrayBuffer>, { status: 200, headers: answer.headers }) : refused(JSON.parse(notServed()) as Answer & { kind: "refused" });
}

/** A pack's file: the bundle's, or the store's for a pack the bundle doesn't hold. */
async function packFile(bundle: Bundle, store: Store | undefined, place: Place): Promise<Uint8Array | undefined> {
  return (await bundle.packFile(place)) ?? (await store?.file(place));
}

/** A refusal, saying why in plain text, to any origin. */
function refused({ status, why, headers }: Answer & { kind: "refused" }): Response {
  return new Response(why, { status, headers });
}
