// A registry, and npm's protocol (docs/resolving-registry-packs.md, decision 7): a pack's versions from its short
// metadata, and a version's tarball. Checking and unpacking are the resolver's, so no registry can skip them.

/** A version of a pack, as a registry's metadata lists it. */
export interface VersionInfo {
  version: string;
  /** Where its tarball is. */
  tarball: string;
  /** The integrity the registry publishes for the tarball, `sha512-…`. */
  integrity?: string;
  /** Its unpacked size in bytes, which a host's size cap is checked against before anything is downloaded. */
  unpackedSize?: number;
}

/** A pack's versions, and for how long the list may be kept, in seconds. */
export interface Versions {
  versions: VersionInfo[];
  maxAge: number;
}

/** Where packs come from. */
export interface Registry {
  /** Its URL: what a cache keeps its versions lists by, and what `fetch: "host"` checks. */
  readonly url: string;
  /** Every version of the pack `name`; none when the registry doesn't have it. */
  versions(name: string): Promise<Versions>;
  /** The bytes of a version's tarball, unchecked. */
  tarball(version: VersionInfo): Promise<Uint8Array<ArrayBuffer>>;
}

/** The short form of npm's metadata, which lists every version with its tarball and integrity. */
const SHORT = "application/vnd.npm.install-v1+json";

/** A scoped npm name, which is all a pack's name may be (spec §6.4); anything else never reaches a URL. */
const SCOPED = /^@[a-z0-9~-][a-z0-9._~-]*\/[a-z0-9~-][a-z0-9._~-]*$/;

/** Whether `name` is a pack's name: scoped, in npm's characters. */
export const isPackName = (name: string): boolean => SCOPED.test(name);

/** The URL of the page this runs in; `undefined` on a server or in a worker without one. */
export const pageHref = (): string | undefined => (globalThis as { location?: { href: string } }).location?.href;

/** The seconds of `max-age` in a `Cache-Control` header; 0 without one. */
export function maxAgeOf(cacheControl: string | null): number {
  const match = /(?:^|,)\s*max-age=(\d+)/.exec(cacheControl ?? "");
  return match ? Number(match[1]) : 0;
}

export interface NpmRegistryOptions {
  /** The registry, `https://registry.npmjs.org/`, or a host's proxy, `/compmd/registry/` (relative to the page). */
  url: string;
  /** The `fetch` to use, for a test or a host that adds what its registry needs. */
  fetch?: typeof fetch;
  /** Headers sent with each request, such as a private registry's token; only on a server or in a proxy. */
  headers?: Record<string, string>;
}

/** npm's metadata of a pack, as far as a resolver reads it. */
interface Packument {
  versions?: Record<string, { dist?: { tarball?: unknown; integrity?: unknown; unpackedSize?: unknown } }>;
}

/**
 * A registry that speaks npm's protocol: npm's own, a mirror, a private registry, a proxy.
 *
 * @example
 * const npm = new NpmRegistry({ url: "https://registry.npmjs.org/" });
 * const { versions } = await npm.versions("@kit/mermaid");
 */
export class NpmRegistry implements Registry {
  readonly url: string;
  readonly #fetch: typeof fetch;
  readonly #headers: Record<string, string>;

  /** @throws Error when `url` is relative and there is no page to resolve it against. */
  constructor({ url, fetch: fetcher = globalThis.fetch.bind(globalThis), headers = {} }: NpmRegistryOptions) {
    const base = new URL(url, pageHref()).href;
    this.url = base.endsWith("/") ? base : `${base}/`;
    this.#fetch = fetcher;
    this.#headers = headers;
  }

  /**
   * The registry's short metadata of `name`, as it answered, with how long it may be kept; `undefined` when it
   * doesn't have the pack.
   *
   * @throws Error when `name` isn't a pack's name, or the registry fails.
   */
  async metadata(name: string): Promise<{ body: string; maxAge: number } | undefined> {
    if (!isPackName(name)) throw new Error(`\`${name}\` isn't a pack's name`);
    const response = await this.#get(`${this.url}${name.replace("/", "%2f")}`, SHORT);
    if (response.status === 404) return undefined;
    if (!response.ok) throw new Error(`${this.url} answered ${String(response.status)} for \`${name}\``);
    return { body: await response.text(), maxAge: maxAgeOf(response.headers.get("cache-control")) };
  }

  async versions(name: string): Promise<Versions> {
    const metadata = await this.metadata(name);
    if (!metadata) return { versions: [], maxAge: 0 };
    const packument = JSON.parse(metadata.body) as Packument;
    const versions = Object.entries(packument.versions ?? {}).flatMap(([version, { dist }]): VersionInfo[] => {
      if (typeof dist?.tarball !== "string") return [];
      return [
        {
          version,
          tarball: new URL(dist.tarball, this.url).href,
          ...(typeof dist.integrity === "string" ? { integrity: dist.integrity } : {}),
          ...(typeof dist.unpackedSize === "number" ? { unpackedSize: dist.unpackedSize } : {}),
        },
      ];
    });
    return { versions, maxAge: metadata.maxAge };
  }

  /** @throws Error when the registry fails. */
  async tarball(version: VersionInfo): Promise<Uint8Array<ArrayBuffer>> {
    return new Uint8Array(await (await this.tarballResponse(version)).arrayBuffer());
  }

  /**
   * The registry's response for a version's tarball, its body unread, for a proxy to stream on.
   *
   * @throws Error when the registry fails.
   */
  async tarballResponse({ tarball }: VersionInfo): Promise<Response> {
    const response = await this.#get(tarball, "application/octet-stream");
    if (!response.ok) throw new Error(`${tarball} answered ${String(response.status)}`);
    return response;
  }

  /** Every request without credentials, carrying only a pack's name and version (companion, section 4). */
  #get(url: string, accept: string): Promise<Response> {
    return this.#fetch(url, { headers: { ...this.#headers, accept }, credentials: "omit" });
  }
}
