// Resolving a request's packs from registries (docs/resolving-registry-packs.md; the companion compmd-registry-v1.md,
// sections 3 to 5) by the registry rules, `compmd-host`'s, as WebAssembly (docs/registry-rules.md): the rules decide
// everything, the version, the policy before any download, the checks of a tarball and its reading; this drives their
// steps with the host's registries, its cache and the processor's `sources`, each round's steps at once, and gives a
// pack up after its timeout.

import { loaded, Resolution, type Answer, type Files, type Resolved as Rules, type Step } from "@compmd/host/registry";
import { sources, type HostComponent, type NeededPack, type PackRelease, type Request } from "@compmd/processor";
import { memoryCache, type PackCache } from "./cache.ts";
import { messageOf } from "./message-of.ts";
import { pageHref, type Registry } from "./registry.ts";

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
   * A file a frame may load of a pack this resolve fetched, checked: an image or a font's file (woff2, woff, ttf, otf),
   * by its place (`@acme/kit@1.4.2/icons/star.svg`), as the expanded tree and the fonts name it (spec §12.3, §9.4), for
   * the sandbox; `undefined` for any other file. A pack's other files are never given: opened from a `blob:` URL, a
   * page or a script would run with the host's page's origin. (`images` until 0.11.0.)
   */
  files: (place: string) => Blob | undefined;
  problems: Problem[];
}

/** The origin of the page the resolver runs in, by its URL (an opaque-origin page's too); `undefined` on a server. */
function pageOrigin(): string | undefined {
  const href = pageHref();
  return href === undefined ? undefined : new URL(href).origin;
}

/**
 * The packs `request`'s document needs, fetched from `options.registries` and checked: the document's imports that the
 * request has no release of, then those of the packs fetched, until none is missing or none can be got. The
 * processor's module must be loaded first (`init` from `@compmd/processor`); the rules' loads itself.
 *
 * @example
 * const registries = { "*": new NpmRegistry({ url: "/compmd/registry/" }) }; // the host's proxy
 * const { request: ready, modules } = await resolvePacks(request, { registries, cache: defaultCache() });
 * const processed = process(ready);
 *
 * @throws Error when `fetch` is `host` in a browser and a registry is on another origin.
 */
export async function resolvePacks(request: Request, options: ResolveOptions): Promise<Resolved> {
  const driver = new Driver(request, options);
  await loaded();
  const resolved = await driver.run();
  return {
    request: driver.request,
    modules: ({ pack, version, path }) => {
      const code = resolved.modules[`${pack}@${version}/${path}`];
      return code === undefined ? undefined : { code: new Blob([code], { type: "text/javascript" }) };
    },
    files: (place) => {
      const file = resolved.files[place];
      return file && new Blob([new Uint8Array(file.bytes)], { type: file.mediaType });
    },
    problems: resolved.problems,
  };
}

/** A step being done. */
type Doing = Promise<void>;

/** Drives one resolution: does its steps, gives their answers, and gives packs up that don't come in time. */
class Driver {
  request: Request;
  readonly #options: ResolveOptions;
  readonly #cache: PackCache;
  readonly #timeout: number;
  /** In a browser with `fetch: "host"`, the only origin requests may go to. */
  readonly #origin: string | undefined;
  /** The host's registries by the name the rules know each by: its URL, or with `#n` after it when two share one. */
  readonly #byName = new Map<string, Registry>();
  /** Each registry's name for the rules. */
  readonly #names = new Map<Registry, string>();
  /** The steps being done. */
  readonly #doing = new Set<Doing>();
  /** The timers of the round's packs. */
  #timers: ReturnType<typeof setTimeout>[] = [];
  /** Ends the wait for the next thing done: a step, or a pack given up. */
  #wake: () => void = () => undefined;
  #resolution: Resolution | undefined;

  /** @throws Error when a registry is on another origin than the page's, with `fetch: "host"`. */
  constructor(request: Request, options: ResolveOptions) {
    this.request = { ...request, packs: { ...request.packs }, policy: { ...request.policy } };
    this.#options = options;
    this.#cache = options.cache ?? memoryCache();
    this.#timeout = options.timeout ?? DEFAULT_TIMEOUT;
    this.#origin = (options.fetch ?? "host") === "host" ? pageOrigin() : undefined;
    for (const registry of Object.values(options.registries)) {
      this.#allow(registry.url);
      if (this.#names.has(registry)) continue;
      const name = this.#byName.has(registry.url) ? `${registry.url}#${String(this.#names.size)}` : registry.url;
      this.#byName.set(name, registry);
      this.#names.set(registry, name);
    }
  }

  /** What the resolution gives, once it is done. */
  async run(): Promise<Rules> {
    const registries = Object.fromEntries(Object.entries(this.#options.registries).map(([scope, registry]) => [scope, this.#names.get(registry) ?? registry.url]));
    const deny = (this.request.policy?.deny ?? []).map(({ pack, version }) => (version === undefined ? { pack } : { pack, version }));
    const resolution = new Resolution({ registries, maxPackSize: this.#options.maxPackSize ?? DEFAULT_MAX_PACK_SIZE, deny });
    this.#resolution = resolution;
    try {
      for (;;) {
        const next = resolution.next();
        if ("done" in next) return next.done;
        const woken = new Promise<void>((resolve) => (this.#wake = resolve));
        for (const step of next.steps) this.#start(step);
        await woken;
      }
    } finally {
      for (const timer of this.#timers) clearTimeout(timer);
    }
  }

  /** Does `step`, and gives its answer when it comes. */
  #start(step: Step): void {
    const doing: Doing = this.#perform(step)
      .then(
        (answer) => {
          if (answer !== undefined) this.#resolution?.give(step.id, answer);
        },
        (error: unknown) => {
          this.#resolution?.give(step.id, { kind: "failed", message: messageOf(error) });
        },
      )
      .finally(() => {
        this.#doing.delete(doing);
        this.#wake();
      });
    this.#doing.add(doing);
  }

  /** What `step` answers; `undefined` when it was answered already (a tarball's bytes). */
  async #perform(step: Step): Promise<Answer | undefined> {
    switch (step.kind) {
      case "missing":
        return this.#missing(step.releases, step.refused);
      case "keptVersions": {
        const kept = await this.#cache.versions(`${this.#registry(step.registry).url}${step.pack}`);
        return { kind: "keptVersions", kept: kept ?? null };
      }
      case "versions": {
        const { versions, maxAge } = await this.#registry(step.registry).versions(step.pack);
        return { kind: "versions", versions, maxAge };
      }
      case "keepVersions":
        await this.#cache.keepVersions(`${this.#registry(step.registry).url}${step.pack}`, step.versions, step.maxAge);
        return { kind: "done" };
      case "kept":
        return { kind: "kept", files: (await this.#kept(step.release)) ?? null };
      case "tarball": {
        this.#allow(step.version.tarball);
        const bytes = await this.#registry(step.registry).tarball(step.version, step.maxBytes);
        this.#resolution?.giveTarball(step.id, bytes);
        return undefined;
      }
      case "keep":
        await this.#cache.keep(step.release, step.files.text);
        await this.#cache.keepBinary?.(step.release, binaryOf(step.files));
        return { kind: "done" };
    }
  }

  /** The releases and refusals added to the request, and what it still imports, each pack timed from now. */
  #missing(releases: [string, PackRelease][], refused: Record<string, string>): Answer {
    const packs = { ...this.request.packs };
    for (const [pack, release] of releases) {
      const known = packs[pack] ?? [];
      if (!known.some(({ version }) => version === release.version)) packs[pack] = [...known, release];
    }
    this.request = { ...this.request, packs, policy: { ...this.request.policy, refused: { ...this.request.policy?.refused, ...refused } } };
    for (const timer of this.#timers) clearTimeout(timer);
    const needed = sources(this.request).packs;
    this.#timers = [...new Set(needed.map(({ pack }) => pack))].map((pack) =>
      setTimeout(() => {
        this.#resolution?.giveUp(pack, this.#timeout);
        this.#wake();
      }, this.#timeout),
    );
    return { kind: "missing", packs: needed.map(neededOf) };
  }

  /**
   * The files the cache keeps of `release`: its text, and its binary files when the cache keeps them. A cache that
   * keeps them but has none for it kept the text before it did, so the tarball is fetched again.
   */
  async #kept(release: { pack: string; version: string; integrity: string }): Promise<Files | undefined> {
    const text = await this.#cache.files(release);
    if (!text) return undefined;
    if (!this.#cache.binaryFiles) return { text, binary: {} };
    const binary = await this.#cache.binaryFiles(release);
    if (!binary) return undefined;
    return { text, binary: Object.fromEntries(Object.entries(binary).map(([path, bytes]) => [path, [...bytes]])) };
  }

  /** The registry the rules name `name`. */
  #registry(name: string): Registry {
    const registry = this.#byName.get(name);
    if (!registry) throw new Error(`no registry is named ${name}`);
    return registry;
  }

  /** @throws Error when `url` is on another origin than the page's, with `fetch: "host"`. */
  #allow(url: string): void {
    if (this.#origin === undefined || new URL(url).origin === this.#origin) return;
    throw new Error(`\`${url}\` isn't on this page's origin, and \`fetch\` is \`host\`: the reader's browser fetches from a registry only with \`fetch: "direct"\``);
  }
}

/** A pack the processor lists, as the rules take it. */
function neededOf({ pack, range, integrity, library }: NeededPack): { pack: string; range: string; integrity?: string; library: boolean } {
  return { pack, range, ...(integrity === undefined ? {} : { integrity }), library: library ?? false };
}

/** A release's binary files as bytes. */
function binaryOf({ binary }: Files): Record<string, Uint8Array> {
  return Object.fromEntries(Object.entries(binary).map(([path, bytes]) => [path, new Uint8Array(bytes)]));
}
