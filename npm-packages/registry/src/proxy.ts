// The host's proxy (docs/resolving-registry-packs.md, decision 9; design R1, F1): a fetch handler that passes a pack's
// metadata and tarballs on from the host's registries, so the registry never learns who reads which document. It
// answers nothing else, passes on no credentials, and caches nothing itself.

import { isPackName, NpmRegistry } from "./registry.ts";

export interface ProxyOptions {
  /** The registries by scope, `@kit`, with `*` for every other scope. */
  registries: Readonly<Record<string, NpmRegistry>>;
  /** The path the proxy is served at, ending in `/`: `/compmd/registry/` (the default `/`). */
  base?: string;
}

/** npm's metadata of a pack, as far as the proxy rewrites it. */
interface Packument {
  versions?: Record<string, { dist?: { tarball?: unknown } }>;
}

const notFound = (): Response => new Response("Not found", { status: 404, headers: { "content-type": "text/plain" } });

/**
 * A proxy for packs: `GET <base><name>` gives the pack's short metadata from its scope's registry, with each tarball's
 * URL rewritten to `GET <base>-/tarball/<name>/<version>`, which streams that tarball. Everything else is 404.
 *
 * @example
 * // A worker:
 * export default { fetch: registryProxy({ registries: { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) } }) };
 */
export function registryProxy({ registries, base = "/" }: ProxyOptions): (request: Request) => Promise<Response> {
  const registryOf = (name: string): NpmRegistry | undefined => registries[name.slice(0, name.indexOf("/"))] ?? registries["*"];
  return async (request) => {
    const url = new URL(request.url);
    if (request.method !== "GET" || !url.pathname.startsWith(base)) return notFound();
    const rest = decodeURIComponent(url.pathname.slice(base.length));
    const tarball = /^-\/tarball\/(@[^/]+\/[^/]+)\/([^/]+)$/.exec(rest);
    const name = tarball ? tarball[1] : rest;
    const registry = name === undefined || !isPackName(name) ? undefined : registryOf(name);
    if (name === undefined || !registry) return notFound();
    try {
      if (tarball) return await passTarball(registry, name, tarball[2] ?? "");
      return await passMetadata(registry, name, `${url.origin}${base}`);
    } catch {
      return new Response("Bad gateway", { status: 502, headers: { "content-type": "text/plain" } });
    }
  };
}

/** The pack's short metadata, each tarball's URL the proxy's own. */
async function passMetadata(registry: NpmRegistry, name: string, base: string): Promise<Response> {
  const metadata = await registry.metadata(name);
  if (!metadata) return notFound();
  const packument = JSON.parse(metadata.body) as Packument;
  for (const [version, { dist }] of Object.entries(packument.versions ?? {})) {
    if (dist) dist.tarball = `${base}-/tarball/${name}/${encodeURIComponent(version)}`;
  }
  return new Response(JSON.stringify(packument), {
    headers: { "content-type": "application/vnd.npm.install-v1+json", "cache-control": `public, max-age=${String(metadata.maxAge)}` },
  });
}

/** The tarball of `name` at `version`, from where the registry's metadata says it is. */
async function passTarball(registry: NpmRegistry, name: string, version: string): Promise<Response> {
  const info = (await registry.versions(name)).versions.find((known) => known.version === version);
  if (!info) return notFound();
  const upstream = await registry.tarballResponse(info);
  // A version's tarball never changes.
  return new Response(upstream.body, { headers: { "content-type": "application/octet-stream", "cache-control": "public, max-age=31536000, immutable" } });
}
