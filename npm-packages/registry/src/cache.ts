// What the resolver fetched, kept by what can't change (docs/resolving-registry-packs.md, decision 10; design R4): a
// release's files by its pack, version and integrity, which never change, and a pack's versions list for as long as
// its registry says.

import type { VersionInfo } from "./registry.ts";
import { releaseKey, type ReleaseId } from "./release-id.ts";

/** A versions list kept, and whether it is still within its registry's `max-age`. */
export interface KeptVersions {
  versions: VersionInfo[];
  fresh: boolean;
}

/** Where the resolver keeps what it fetched. */
export interface PackCache {
  /**
   * The text files of `release`'s checked tarball, if kept: only those kept for the same pack, version and integrity,
   * so one pack's files are never given as another's.
   */
  files(release: ReleaseId): Promise<Record<string, string> | undefined>;
  keep(release: ReleaseId, files: Record<string, string>): Promise<void>;
  /**
   * The files that aren't text of `release`'s checked tarball (its images), if kept, as `files` finds them. A cache
   * without it keeps text only: a pack it gives has no images.
   */
  binaryFiles?(release: ReleaseId): Promise<Record<string, Uint8Array> | undefined>;
  keepBinary?(release: ReleaseId, files: Record<string, Uint8Array>): Promise<void>;
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
  const binary = new Map<string, Record<string, Uint8Array>>();
  const lists = new Map<string, Stored>();
  return {
    files: (release) => Promise.resolve(files.get(releaseKey(release))),
    keep: (release, text) => {
      files.set(releaseKey(release), text);
      return Promise.resolve();
    },
    binaryFiles: (release) => Promise.resolve(binary.get(releaseKey(release))),
    keepBinary: (release, bytes) => {
      binary.set(releaseKey(release), bytes);
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

/** The Cache Storage cache's name; a new layout gets a new name (v2: files by release, no longer by integrity alone). */
export const CACHE_NAME = "compmd-registry-v2";

/** A Cache Storage key: a URL that never reaches a network. */
const keyOf = (kind: "files" | "binary" | "versions", key: string): string => `https://compmd.invalid/${kind}/${encodeURIComponent(key)}`;

/** The bytes of `files`, each as base64, which JSON holds. */
const toBase64 = (files: Record<string, Uint8Array>): Record<string, string> => Object.fromEntries(Object.entries(files).map(([path, bytes]) => [path, btoa(Array.from(bytes, (byte) => String.fromCharCode(byte)).join(""))]));

/** `files` written as base64, as bytes again. */
const fromBase64 = (files: Record<string, string>): Record<string, Uint8Array> => Object.fromEntries(Object.entries(files).map(([path, text]) => [path, Uint8Array.from(atob(text), (char) => char.charCodeAt(0))]));

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
    files: async (release) => {
      const found = (await memory.files(release)) ?? (await read<Record<string, string>>(await storage, keyOf("files", releaseKey(release))));
      if (found) await memory.keep(release, found);
      return found;
    },
    keep: async (release, files) => {
      await memory.keep(release, files);
      await write(await storage, keyOf("files", releaseKey(release)), files);
    },
    binaryFiles: async (release) => {
      const inMemory = await memory.binaryFiles?.(release);
      if (inMemory) return inMemory;
      const stored = await read<Record<string, string>>(await storage, keyOf("binary", releaseKey(release)));
      const found = stored && fromBase64(stored);
      if (found) await memory.keepBinary?.(release, found);
      return found;
    },
    keepBinary: async (release, files) => {
      await memory.keepBinary?.(release, files);
      await write(await storage, keyOf("binary", releaseKey(release)), toBase64(files));
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
