// Resolving a request's packs from registries (docs/resolving-registry-packs.md, decisions 1 to 6 and 8; the companion
// compmd-registry-v1.md, sections 3 to 5): ask the processor which packs are missing, resolve each range as the
// processor does (a library pack's to the version every range asked for it matches, docs/sharing-library-packs.md,
// decision 5), apply the policy before downloading, check each tarball, add the releases, and ask again.

import { newestMatching, sources, type HostComponent, type NeededPack, type PackRelease, type Policy, type Request } from "@compmd/processor";
import { memoryCache, type PackCache } from "./cache.ts";
import { imageTypeOf } from "./image-types.ts";
import { integrityOf, isChecked } from "./integrity.ts";
import { pageHref, type Registry, type VersionInfo, type Versions } from "./registry.ts";
import { releaseKey, type ReleaseId } from "./release-id.ts";
import { binaryFiles, gunzip, textFiles, untar } from "./tar.ts";

/** A version's checked files: its text, which the processor reads, and the rest as bytes, its images among them. */
interface Checked {
  text: Record<string, string>;
  binary: Record<string, Uint8Array>;
}

/** The size cap when the host sets none: Mermaid 12's module as one file is 7 MB, with its layout engines. */
export const DEFAULT_MAX_PACK_SIZE = 10_000_000;

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
  /** The largest unpacked size of a pack, in bytes; 10 MB when not given. */
  maxPackSize?: number;
  /**
   * How long a pack may take to come, in ms (30 s when not given): one that hasn't is a problem, and stays missing, so
   * a host that shows it as pending meanwhile (spec §6.4) stops, and the processor reports F026.
   */
  timeout?: number;
}

/** How long a pack may take when the host sets nothing. */
export const DEFAULT_TIMEOUT = 30_000;

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
  /**
   * An image of a pack this resolve fetched, checked, by its place (`@acme/kit@1.4.2/icons/star.svg`), as the expanded
   * tree names it (spec §12.3), for the sandbox to show; `undefined` for any other file. A pack's other files are never
   * given: opened from a `blob:` URL, a page or an SVG would run with the host's page's origin.
   */
  images: (place: string) => Blob | undefined;
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
    for (const needed of missing) {
      asked.add(keyOf(needed));
      resolver.ask(needed);
    }
    const outcomes = await Promise.all(missing.map((needed) => resolver.resolve(needed, current.policy)));
    current = withOutcomes(current, missing, outcomes, resolver.problems);
  }
  return { request: current, modules: (place) => resolver.module(place), images: (place) => resolver.image(place), problems: resolver.problems };
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
  readonly #timeout: number;
  /** In a browser with `fetch: "host"`, the only origin requests may go to. */
  readonly #origin: string | undefined;
  /** The text of every built module fetched, by its place, `pack@version/path`. */
  readonly #modules = new Map<string, string>();
  /** Every image fetched, by its place. */
  readonly #images = new Map<string, Blob>();
  /** Every range asked for each library pack in this resolve, of which the processor takes the version all match. */
  readonly #libraryRanges = new Map<string, string[]>();
  /** The files of each release being got, by its `releaseKey`. */
  readonly #fetching = new Map<string, Promise<Checked | undefined>>();

  constructor({ registries, fetch = "host", cache = memoryCache(), maxPackSize = DEFAULT_MAX_PACK_SIZE, timeout = DEFAULT_TIMEOUT }: ResolveOptions) {
    this.#registries = registries;
    this.#cache = cache;
    this.#maxSize = maxPackSize;
    this.#timeout = timeout;
    this.#origin = fetch === "host" ? pageOrigin() : undefined;
    for (const registry of Object.values(registries)) this.#allow(registry.url);
  }

  /** Notes a pack the processor lists, before any is resolved, so a library's ranges asked in one round are known together. */
  ask({ pack, range, library }: NeededPack): void {
    if (!library) return;
    const ranges = this.#libraryRanges.get(pack) ?? [];
    if (!ranges.includes(range)) this.#libraryRanges.set(pack, [...ranges, range]);
  }

  module({ pack, version, path }: ModulePlace): { code: Blob } | undefined {
    const text = this.#modules.get(`${pack}@${version}/${path}`);
    return text === undefined ? undefined : { code: new Blob([text], { type: "text/javascript" }) };
  }

  image(place: string): Blob | undefined {
    return this.#images.get(place);
  }

  /** One missing pack: its release, or why it isn't there, a registry that didn't answer in time among them. */
  async resolve(needed: NeededPack, policy: Policy | undefined): Promise<Outcome> {
    const { pack } = needed;
    const registry = this.#registries[pack.slice(0, pack.indexOf("/"))] ?? this.#registries["*"];
    if (!registry) return { problem: `no registry is configured for \`${pack}\`'s scope` };
    let timer: ReturnType<typeof setTimeout> | undefined;
    const late = new Promise<Outcome>((resolve) => {
      timer = setTimeout(() => {
        resolve({ problem: `\`${pack}\` didn't come within ${String(this.#timeout / 1000)} s` });
      }, this.#timeout);
    });
    try {
      return await Promise.race([this.#resolveFrom(registry, needed, policy), late]);
    } catch (error) {
      return { problem: messageOf(error) };
    } finally {
      clearTimeout(timer);
    }
  }

  async #resolveFrom(registry: Registry, { pack, range, integrity: pinned, library }: NeededPack, policy: Policy | undefined): Promise<Outcome> {
    // A pinned import names an exact version, and its files never change: kept for this pack, they need no registry,
    // and the host's policy holds for them as for a download.
    if (pinned !== undefined) {
      const kept = await this.#kept({ pack, version: range, integrity: pinned });
      if (kept) {
        const info = { version: range, tarball: "", integrity: pinned, unpackedSize: unpackedSizeOf(kept) };
        return this.#withheld(pack, info, policy) ?? this.#release(pack, info, kept);
      }
    }
    const versions = await this.#versions(registry, pack);
    const known = versions.map(({ version }) => version);
    const version = (library && this.#everyRange(pack, known)) || newestMatching(range, known);
    const info = versions.find((known) => known.version === version);
    // None matching stays F026 (no release matches), in the processor.
    if (!info) return undefined;
    // A pinned integrity the registry's doesn't equal is the processor's F035: it checks it before it reads a pack, so
    // the release needs no files.
    if (pinned !== undefined && pinned !== info.integrity) return filelessOf(info);
    const withheld = this.#withheld(pack, info, policy);
    if (withheld) return withheld;
    const files = await this.#filesOf(registry, { pack, version: info.version, integrity: info.integrity as string }, info);
    if (!files) return { release: { ...filelessOf(info).release, broken: "its tarball doesn't match the integrity its registry publishes" } };
    return this.#release(pack, info, files);
  }

  /**
   * What the host's policy gives instead of `info`'s files, `undefined` when it takes them: a denied version is the
   * processor's F026, which it checks before it reads a pack, so the release needs no files; a refused one is listed in
   * `policy.refused`.
   */
  #withheld(pack: string, info: VersionInfo, policy: Policy | undefined): Outcome | undefined {
    if (denied(policy, pack, info.version)) return filelessOf(info);
    const refusal = this.#refusal(pack, info);
    return refusal ? { refused: refusal } : undefined;
  }

  /**
   * The checked files of `release`, from the cache or its tarball (`info`'s), fetched once in a resolve however many
   * imports resolve to it at once; `undefined` when the tarball doesn't match its integrity.
   */
  #filesOf(registry: Registry, release: ReleaseId, info: VersionInfo): Promise<Checked | undefined> {
    const key = releaseKey(release);
    let files = this.#fetching.get(key);
    if (!files) {
      files = this.#fetchFiles(registry, release, info);
      this.#fetching.set(key, files);
    }
    return files;
  }

  /**
   * The files the cache keeps of `release`: its text, and its binary files when the cache keeps them. A cache that
   * keeps them but has none for it kept the text before it did, so the tarball is fetched again.
   */
  async #kept(release: ReleaseId): Promise<Checked | undefined> {
    const text = await this.#cache.files(release);
    if (!text) return undefined;
    if (!this.#cache.binaryFiles) return { text, binary: {} };
    const binary = await this.#cache.binaryFiles(release);
    return binary ? { text, binary } : undefined;
  }

  async #fetchFiles(registry: Registry, release: ReleaseId, info: VersionInfo): Promise<Checked | undefined> {
    const kept = await this.#kept(release);
    if (kept) return kept;
    this.#allow(info.tarball);
    const tarball = await registry.tarball(info);
    if ((await integrityOf(tarball)) !== release.integrity) return undefined;
    const files = untar(await gunzip(tarball));
    const checked = { text: textFiles(files), binary: binaryFiles(files) };
    await this.#cache.keep(release, checked.text);
    await this.#cache.keepBinary?.(release, checked.binary);
    return checked;
  }

  /**
   * The newest of `versions` that every range asked for the library `pack` matches, as the processor reads ranges
   * (docs/sharing-library-packs.md, decision 5); `undefined` when none does.
   */
  #everyRange(pack: string, versions: string[]): string | undefined {
    const ranges = this.#libraryRanges.get(pack) ?? [];
    const matching = versions.filter((version) => ranges.every((range) => newestMatching(range, [version]) === version));
    return newestMatching("*", matching);
  }

  /** Why the host's policy refuses `info` before it is downloaded: its size (companion, section 3, step 3), or no hash to check it by. */
  #refusal(pack: string, { version, integrity, unpackedSize }: VersionInfo): string | undefined {
    const cap = `${String(this.#maxSize / MB)} MB`;
    if (unpackedSize === undefined) return `its registry doesn't say how large \`${pack}@${version}\` is, and this host takes packs up to ${cap}`;
    if (unpackedSize > this.#maxSize) return `\`${pack}@${version}\` is ${(unpackedSize / MB).toFixed(1)} MB unpacked, more than this host's ${cap}`;
    if (!isChecked(integrity)) return `its registry publishes no SHA-512 integrity for \`${pack}@${version}\`, so it can't be checked`;
    return undefined;
  }

  /** The release of `info` with its checked files, keeping its built modules and its images. */
  #release(pack: string, info: VersionInfo, { text, binary }: Checked): Outcome {
    const placeOf = (path: string) => `${pack}@${info.version}/${path}`;
    for (const [path, code] of Object.entries(text)) {
      if (path.endsWith(".js")) this.#modules.set(placeOf(path), code);
    }
    for (const [path, content] of [...Object.entries(text), ...Object.entries(binary)]) {
      const type = imageTypeOf(path);
      if (type !== undefined) this.#images.set(placeOf(path), new Blob([content as BlobPart], { type }));
    }
    return { release: { version: info.version, integrity: info.integrity ?? null, files: text } };
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

/** `info` as a release with no files, for a version the processor refuses before it reads a pack. */
const filelessOf = (info: VersionInfo): { release: PackRelease } => ({ release: { version: info.version, integrity: info.integrity ?? null, files: {} } });

/** The unpacked size of kept files, in bytes, as a registry's `unpackedSize` counts it: each file's bytes. */
function unpackedSizeOf({ text, binary }: Checked): number {
  const encoder = new TextEncoder();
  const textSize = Object.values(text).reduce((size, content) => size + encoder.encode(content).length, 0);
  return Object.values(binary).reduce((size, content) => size + content.length, textSize);
}

/**
 * Whether the host's policy denies `pack` at `version`, its `deny` ranges read as an import's are. The processor reads
 * a bare version there more widely (`1.4.2` as `^1.4.2`); a version only it denies is downloaded for nothing, and still
 * denied.
 */
function denied(policy: Policy | undefined, pack: string, version: string): boolean {
  return (policy?.deny ?? []).some((deny) => deny.pack === pack && (deny.version === undefined || newestMatching(deny.version, [version]) === version));
}
