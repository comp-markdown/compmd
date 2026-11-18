// What the resolver fetched, kept by what can't change (docs/resolving-registry-packs.md, decision 10; design R4): a
// version's files by its integrity, which never changes, and a pack's versions list for as long as its registry says.

import type { VersionInfo } from "./registry.ts";

/** A versions list kept, and whether it is still within its registry's `max-age`. */
export interface KeptVersions {
  versions: VersionInfo[];
  fresh: boolean;
}

/** Where the resolver keeps what it fetched. */
export interface PackCache {
  /** The text files of the checked tarball whose integrity is `integrity`, if kept. */
  files(integrity: string): Promise<Record<string, string> | undefined>;
  keep(integrity: string, files: Record<string, string>): Promise<void>;
  /** A pack's versions list kept under `key` (the registry's URL and the pack's name), fresh or stale. */
  versions(key: string): Promise<KeptVersions | undefined>;
  /** Keeps a versions list for `maxAge` seconds; after that it is stale, and used only when the registry can't be reached. */
  keepVersions(key: string, versions: VersionInfo[], maxAge: number): Promise<void>;
}

/** A versions list as kept: until when it is fresh, in ms since 1970. */
interface Stored {
  versions: VersionInfo[];
  until: number;
}

const kept = ({ versions, until }: Stored, now: number): KeptVersions => ({ versions, fresh: now < until });

/**
 * A cache in memory, for the page's life.
 *
 * @param now The clock, for tests.
 */
export function memoryCache(now: () => number = Date.now): PackCache {
  const files = new Map<string, Record<string, string>>();
  const lists = new Map<string, Stored>();
  return {
    files: (integrity) => Promise.resolve(files.get(integrity)),
    keep: (integrity, text) => {
      files.set(integrity, text);
      return Promise.resolve();
    },
    versions: (key) => {
      const stored = lists.get(key);
      return Promise.resolve(stored && kept(stored, now()));
    },
    keepVersions: (key, versions, maxAge) => {
      lists.set(key, { versions, until: now() + maxAge * 1000 });
      return Promise.resolve();
    },
  };
}

/** The Cache Storage cache's name; a new layout gets a new name. */
const CACHE_NAME = "compmd-registry-v1";

/** A Cache Storage key: a URL that never reaches a network. */
const keyOf = (kind: "files" | "versions", key: string): string => `https://compmd.invalid/${kind}/${encodeURIComponent(key)}`;

/** Cache Storage, or `undefined` where the page has none: an opaque origin, a private window that refuses it. */
async function openStorage(): Promise<Cache | undefined> {
  try {
    return await caches.open(CACHE_NAME);
  } catch {
    return undefined;
  }
}

/** The JSON kept under `url`, or `undefined` when there is none or it can't be read. */
async function read<T>(storage: Cache | undefined, url: string): Promise<T | undefined> {
  try {
    const response = await storage?.match(url);
    return response ? ((await response.json()) as T) : undefined;
  } catch {
    return undefined;
  }
}

/** Keeps `value` under `url`; a full or refused storage keeps nothing, and memory still has it. */
async function write(storage: Cache | undefined, url: string, value: unknown): Promise<void> {
  try {
    await storage?.put(url, new Response(JSON.stringify(value), { headers: { "content-type": "application/json" } }));
  } catch {
    // Memory has it for the page's life.
  }
}

/**
 * The default cache: memory for the page's life, and Cache Storage across visits and offline, where the page has it;
 * memory only where it hasn't. It belongs to the page's origin: two sites never share one, and a frame never sees it.
 *
 * @param now The clock, for tests.
 */
export function defaultCache(now: () => number = Date.now): PackCache {
  const memory = memoryCache(now);
  const storage = typeof caches === "undefined" ? Promise.resolve(undefined) : openStorage();
  return {
    files: async (integrity) => {
      const found = (await memory.files(integrity)) ?? (await read<Record<string, string>>(await storage, keyOf("files", integrity)));
      if (found) await memory.keep(integrity, found);
      return found;
    },
    keep: async (integrity, files) => {
      await memory.keep(integrity, files);
      await write(await storage, keyOf("files", integrity), files);
    },
    versions: async (key) => {
      const inMemory = await memory.versions(key);
      if (inMemory) return inMemory;
      const stored = await read<Stored>(await storage, keyOf("versions", key));
      return stored && kept(stored, now());
    },
    keepVersions: async (key, versions, maxAge) => {
      await memory.keepVersions(key, versions, maxAge);
      await write(await storage, keyOf("versions", key), { versions, until: now() + maxAge * 1000 });
    },
  };
}
