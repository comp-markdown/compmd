// Resolving a request's packs from registries (docs/resolving-registry-packs.md, decisions 1 to 6 and 8; the companion
// compmd-registry-v1.md, sections 3 and 4): ask the processor which packs are missing, resolve each range as the
// processor does, apply the policy before downloading, check each tarball, add the releases, and ask again.

import { newestMatching, sources, type HostComponent, type NeededPack, type PackRelease, type Policy, type Request } from "@compmd/processor";
import { memoryCache, type PackCache } from "./cache.ts";
import { integrityOf, isChecked } from "./integrity.ts";
import { pageHref, type Registry, type VersionInfo, type Versions } from "./registry.ts";
import { gunzip, textFiles, untar } from "./tar.ts";

/** The size cap when the host sets none: enough for Mermaid's module as one file, with room to spare. */
export const DEFAULT_MAX_PACK_SIZE = 5_000_000;

export interface ResolveOptions {
  /** The host's registries by scope, `@kit`, with `*` for every other scope, as `.npmrc` has them. */
  registries: Readonly<Record<string, Registry>>;
  /**
   * Who fetches (companion, section 4). `host`, the default: in a browser, every registry and tarball must be on the
   * page's own origin, the host's proxy or cache. `direct`: the reader's browser fetches from any `https:` registry,
   * which the host tells its readers.
   */
  fetch?: "host" | "direct";
  /** Where fetched packs are kept; a new memory cache when not given. */
  cache?: PackCache;
  /** The largest unpacked size of a pack, in bytes; 5 MB when not given. */
  maxPackSize?: number;
}

/** Where a built module is in its pack, as the processor gives it with a host component. */
export type ModulePlace = NonNullable<HostComponent["module"]>;

/** Something that couldn't be fetched, for the host's logs; the document is told by the processor's diagnostics. */
export interface Problem {
  pack: string;
  message: string;
}

export interface Resolved {
  /** The request with the fetched releases in `packs`, and the packs the size cap refuses in `policy.refused`. */
  request: Request;
  /** A built module's checked text, for the sandbox; `undefined` for a module this resolve didn't fetch. */
  modules: (place: ModulePlace) => { code: Blob } | undefined;
  problems: Problem[];
}

/** What resolving one missing pack gave: a release for `packs`, a refusal for `policy.refused`, or a problem. */
type Outcome = { release: PackRelease } | { refused: string } | { problem: string } | undefined;

const MB = 1_000_000;

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : String(error));

/** The origin of the page the resolver runs in, by its URL (an opaque-origin page's too); `undefined` on a server. */
function pageOrigin(): string | undefined {
  const href = pageHref();
  return href === undefined ? undefined : new URL(href).origin;
}

/**
 * The packs `request`'s document needs, fetched from `options.registries` and checked: the document's imports that the
 * request has no release of, then those of the packs fetched, until none is missing or none can be got. The
 * processor's module must be loaded first (`init` from `@compmd/processor`).
 *
 * @example
 * const registries = { "*": new NpmRegistry({ url: "/compmd/registry/" }) }; // the host's proxy
 * const { request: ready, modules } = await resolvePacks(request, { registries, cache: defaultCache() });
 * const processed = process(ready);
 *
 * @throws Error when `fetch` is `host` in a browser and a registry is on another origin.
 */
export async function resolvePacks(request: Request, options: ResolveOptions): Promise<Resolved> {
  const resolver = new Resolver(options);
  let current: Request = { ...request, packs: { ...request.packs }, policy: { ...request.policy } };
  const asked = new Set<string>();
  for (;;) {
    const missing = sources(current).packs.filter((needed) => !asked.has(keyOf(needed)));
    if (missing.length === 0) break;
    for (const needed of missing) asked.add(keyOf(needed));
    const outcomes = await Promise.all(missing.map((needed) => resolver.resolve(needed, current.policy)));
    current = withOutcomes(current, missing, outcomes, resolver.problems);
  }
  return { request: current, modules: (place) => resolver.module(place), problems: resolver.problems };
}

/** One import as the processor lists it. */
const keyOf = ({ pack, range, integrity }: NeededPack): string => `${pack}@${range}#${integrity ?? ""}`;

/** `request` with each outcome added: releases to `packs`, refusals to `policy.refused`, problems to `problems`. */
function withOutcomes(request: Request, missing: NeededPack[], outcomes: Outcome[], problems: Problem[]): Request {
  const packs = { ...request.packs };
  const refused = { ...request.policy?.refused };
  missing.forEach(({ pack }, index) => {
    const outcome = outcomes[index];
    if (outcome === undefined) return;
    if ("release" in outcome) {
      const releases = packs[pack] ?? [];
      if (!releases.some(({ version }) => version === outcome.release.version)) packs[pack] = [...releases, outcome.release];
    } else if ("refused" in outcome) {
      refused[pack] = outcome.refused;
    } else {
      problems.push({ pack, message: outcome.problem });
    }
  });
  return { ...request, packs, policy: { ...request.policy, refused } };
}

/** Resolves missing packs, keeping the built modules it fetched. */
class Resolver {
  readonly problems: Problem[] = [];
  readonly #registries: Readonly<Record<string, Registry>>;
  readonly #cache: PackCache;
  readonly #maxSize: number;
  /** In a browser with `fetch: "host"`, the only origin requests may go to. */
  readonly #origin: string | undefined;
  /** The text of every built module fetched, by its place, `pack@version/path`. */
  readonly #modules = new Map<string, string>();

  constructor({ registries, fetch = "host", cache = memoryCache(), maxPackSize = DEFAULT_MAX_PACK_SIZE }: ResolveOptions) {
    this.#registries = registries;
    this.#cache = cache;
    this.#maxSize = maxPackSize;
    this.#origin = fetch === "host" ? pageOrigin() : undefined;
    for (const registry of Object.values(registries)) this.#allow(registry.url);
  }

  module({ pack, version, path }: ModulePlace): { code: Blob } | undefined {
    const text = this.#modules.get(`${pack}@${version}/${path}`);
    return text === undefined ? undefined : { code: new Blob([text], { type: "text/javascript" }) };
  }

  /** One missing pack: its release, or why it isn't there. */
  async resolve(needed: NeededPack, policy: Policy | undefined): Promise<Outcome> {
    const { pack } = needed;
    const registry = this.#registries[pack.slice(0, pack.indexOf("/"))] ?? this.#registries["*"];
    if (!registry) return { problem: `no registry is configured for \`${pack}\`'s scope` };
    try {
      return await this.#resolveFrom(registry, needed, policy);
    } catch (error) {
      return { problem: messageOf(error) };
    }
  }

  async #resolveFrom(registry: Registry, { pack, range, integrity: pinned }: NeededPack, policy: Policy | undefined): Promise<Outcome> {
    // A pinned import names an exact version, and its files never change.
    if (pinned !== undefined) {
      const cached = await this.#cache.files(pinned);
      if (cached) return this.#release(pack, { version: range, tarball: "", integrity: pinned }, cached);
    }
    const versions = await this.#versions(registry, pack);
    const version = newestMatching(
      range,
      versions.map(({ version }) => version),
    );
    const info = versions.find((known) => known.version === version);
    // None matching stays F026 (no release matches), in the processor.
    if (!info) return undefined;
    const fileless = { release: { version: info.version, integrity: info.integrity ?? null, files: {} } };
    // A denied version and a pinned integrity the registry's doesn't equal are the processor's F026 and F035: it checks
    // both before it reads a pack, so the release needs no files.
    if (denied(policy, pack, info.version) || (pinned !== undefined && pinned !== info.integrity)) return fileless;
    const refusal = this.#refusal(pack, info);
    if (refusal) return { refused: refusal };
    const integrity = info.integrity as string;
    const kept = await this.#cache.files(integrity);
    if (kept) return this.#release(pack, info, kept);
    this.#allow(info.tarball);
    const tarball = await registry.tarball(info);
    if ((await integrityOf(tarball)) !== integrity) {
      return { release: { ...fileless.release, broken: "its tarball doesn't match the integrity its registry publishes" } };
    }
    const files = textFiles(untar(await gunzip(tarball)));
    await this.#cache.keep(integrity, files);
    return this.#release(pack, info, files);
  }

  /** Why the host's policy refuses `info` before it is downloaded: its size (companion, section 3, step 3), or no hash to check it by. */
  #refusal(pack: string, { version, integrity, unpackedSize }: VersionInfo): string | undefined {
    const cap = `${String(this.#maxSize / MB)} MB`;
    if (unpackedSize === undefined) return `its registry doesn't say how large \`${pack}@${version}\` is, and this host takes packs up to ${cap}`;
    if (unpackedSize > this.#maxSize) return `\`${pack}@${version}\` is ${(unpackedSize / MB).toFixed(1)} MB unpacked, more than this host's ${cap}`;
    if (!isChecked(integrity)) return `its registry publishes no SHA-512 integrity for \`${pack}@${version}\`, so it can't be checked`;
    return undefined;
  }

  /** The release of `info` with its checked `files`, keeping its built modules. */
  #release(pack: string, info: VersionInfo, files: Record<string, string>): Outcome {
    for (const [path, text] of Object.entries(files)) {
      if (path.endsWith(".js")) this.#modules.set(`${pack}@${info.version}/${path}`, text);
    }
    return { release: { version: info.version, integrity: info.integrity ?? null, files } };
  }

  /** A pack's versions: the cache's while fresh, else the registry's, else a stale list when the registry fails. */
  async #versions(registry: Registry, pack: string): Promise<VersionInfo[]> {
    const key = `${registry.url}${pack}`;
    const kept = await this.#cache.versions(key);
    if (kept?.fresh) return kept.versions;
    let fetched: Versions;
    try {
      fetched = await registry.versions(pack);
    } catch (error) {
      if (kept) return kept.versions;
      throw error;
    }
    await this.#cache.keepVersions(key, fetched.versions, fetched.maxAge);
    return fetched.versions;
  }

  /** @throws Error when `url` is on another origin than the page's, with `fetch: "host"`. */
  #allow(url: string): void {
    if (this.#origin === undefined || new URL(url).origin === this.#origin) return;
    throw new Error(`\`${url}\` isn't on this page's origin, and \`fetch\` is \`host\`: the reader's browser fetches from a registry only with \`fetch: "direct"\``);
  }
}

/**
 * Whether the host's policy denies `pack` at `version`, its `deny` ranges read as an import's are. The processor reads
 * a bare version there more widely (`1.4.2` as `^1.4.2`); a version only it denies is downloaded for nothing, and still
 * denied.
 */
function denied(policy: Policy | undefined, pack: string, version: string): boolean {
  return (policy?.deny ?? []).some((deny) => deny.pack === pack && (deny.version === undefined || newestMatching(deny.version, [version]) === version));
}
