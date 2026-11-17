// The serving contract (docs/hosting.md, decision 8) over the Fetch API: a bundle's frames and the packs the bundle or
// the store holds, with the headers that sandbox them, and a refusal, saying why, for anything else. compmd-host's
// `respond` answers the same, over `http` types; both run the contract's cases (`host/fixtures/respond/`).

import { contentType } from "./content-type.ts";

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

/** The headers every served file has. */
export const SERVED_HEADERS: Readonly<Record<string, string>> = {
  // Sandboxed even when a frame document is opened outside a frame.
  "content-security-policy": "sandbox allow-scripts",
  // A frame's module scripts come from an opaque origin, so they are cross-origin requests.
  "access-control-allow-origin": "*",
  // A page that checks how its frames are served (a host's check) reads these headers.
  "access-control-expose-headers": "*",
  "x-content-type-options": "nosniff",
  // A frame's path names its build, a pack's its version, so what it serves never changes.
  "cache-control": "max-age=31536000, immutable",
};

/** Why a request that isn't for a file the host serves is refused. */
const NOT_SERVED = "not a file of the frames or packs this host serves";

/** Why a request with another method is refused. */
const ONLY_GET = "the frames and packs are only read, with GET";

/**
 * The answer to `request`: the frames' file or pack file it names, from `bundle` or, for a pack the bundle doesn't
 * hold, from `store`; a `405` for another method than `GET`; a `404` for anything else. Each refusal says why, in
 * plain text, readable from any origin.
 *
 * The paths it answers are `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<release>/<path>`, where a
 * path has no empty, `.` or `..` part, so it can't leave its folder. A host mounted under a prefix strips it first.
 *
 * @example
 * // a worker
 * export default { fetch: (request) => respond(request, bundle, store) };
 */
export async function respond(request: Request, bundle: Bundle, store?: Store): Promise<Response> {
  if (request.method !== "GET") return refused(405, ONLY_GET);
  const path = new URL(request.url).pathname;
  const found = await fileOf(bundle, store, path);
  return found ? served(found.path, found.body) : refused(404, NOT_SERVED);
}

/** The bytes of the file `path` names, with the path its type comes from. */
async function fileOf(bundle: Bundle, store: Store | undefined, path: string): Promise<{ body: Uint8Array; path: string } | undefined> {
  if (!path.startsWith("/")) return undefined;
  const [folder, ...parts] = path.slice(1).split("/");
  if (folder === "frames") {
    const [hash, ...file] = parts;
    if (hash !== bundle.hash || !isFilePath(file)) return undefined;
    const body = await bundle.frameFile(file.join("/"));
    return body && { body, path: file.join("/") };
  }
  if (folder === "packs") {
    const [scope, named, ...file] = parts;
    const release = `${scope ?? ""}/${named ?? ""}`;
    if (!isRelease(release) || !isFilePath(file)) return undefined;
    const place = { release, path: file.join("/") };
    const body = (await bundle.packFile(place)) ?? (await store?.file(place));
    return body && { body, path: place.path };
  }
  return undefined;
}

/** Whether `parts` is a file's path in its folder: at least one part, none empty, `.` or `..`. */
const isFilePath = (parts: readonly string[]): boolean => parts.length > 0 && parts.every((part) => part !== "" && part !== "." && part !== "..");

/**
 * Whether `release` has a release's form, `@scope/name@version`. Only one the bundle or the store holds is served, so
 * its name and version are theirs to know.
 */
const isRelease = (release: string): boolean => /^@[^/@]+\/[^/@]+@[^/@]+$/.test(release);

/** `body`, the file at `path`, with its type and the headers. */
function served(path: string, body: Uint8Array): Response {
  return new Response(body as Uint8Array<ArrayBuffer>, { status: 200, headers: { ...SERVED_HEADERS, "content-type": contentType(path) } });
}

/** A refusal with `status`, saying `why` in plain text, to any origin. */
function refused(status: number, why: string): Response {
  return new Response(why, { status, headers: { "content-type": "text/plain; charset=utf-8", "access-control-allow-origin": "*" } });
}
