// Compiled code kept by what was compiled (docs/hosting.md, decision 6): source a reader's frames compile (a document's
// own, a live pack's) is kept by a key over its language, its runtime, the compiler and the source, so the same
// component isn't compiled again in another document or view, and the compile frame isn't even started for it. What
// is kept is still code from a sandbox, never trusted for being kept: it runs where its source would.

import type { Compiled } from "./frame/compile.ts";
import type { CompileRequest } from "./protocol/messages.ts";

export type { Compiled };

/**
 * Where compiled code is kept, by key; the host's to give (Cache Storage on the web by default, an app's data folder
 * on the desktop). A cache that fails, or loses what it kept, is only a miss: the source is compiled again.
 */
export interface CompiledCache {
  get(key: string): Promise<Compiled | undefined>;
  set(key: string, compiled: Compiled): Promise<void>;
}

/**
 * The compiler, as the key has it: the compile step's own version, bumped when `frame/compile.ts` changes what it
 * writes, and the compilers' versions, which a test checks against the package's dependencies. An upgrade changes it,
 * so nothing compiled before is used.
 */
export const COMPILER = "compile/1 sucrase@3.35.1 svelte@5.57.2";

/**
 * The key `request`'s output is kept by: SHA-256, in hex, over its language, its runtime, {@link COMPILER} and its
 * source. Its name isn't in it: the same source compiles the same under any name.
 *
 * @throws Error where the page has no `crypto.subtle` (a page not served securely): then nothing is kept.
 */
export async function compiledKey(request: Pick<CompileRequest, "language" | "runtime" | "code">): Promise<string> {
  const text = JSON.stringify([request.language, request.runtime ?? "", COMPILER, request.code]);
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(digest)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

/** The name of the default cache in Cache Storage. */
const CACHE = "compmd-compiled";

/**
 * A cache in the page's Cache Storage (`caches`), shared by the documents of its origin. Each entry is kept under a
 * URL that is never fetched (`.invalid` never resolves). Where Cache Storage is missing or refuses (a private window,
 * a page not served securely), every lookup misses and nothing is kept.
 *
 * @example
 * const runtime = new HostRuntime({ base, compiled: cacheStorage() }, events);
 */
export function cacheStorage(storage: CacheStorage | undefined = pageCaches(), name = CACHE): CompiledCache {
  const url = (key: string) => `https://compiled.compmd.invalid/${key}`;
  return {
    get: async (key) => {
      try {
        const response = await (await storage?.open(name))?.match(url(key));
        return response ? ((await response.json()) as Compiled) : undefined;
      } catch {
        return undefined;
      }
    },
    set: async (key, compiled) => {
      try {
        await (await storage?.open(name))?.put(url(key), new Response(JSON.stringify(compiled), { headers: { "Content-Type": "application/json" } }));
      } catch {
        // Not kept: compiled again next time.
      }
    },
  };
}

/** The page's Cache Storage; none where it is missing or reading it throws. */
function pageCaches(): CacheStorage | undefined {
  try {
    return globalThis.caches;
  } catch {
    return undefined;
  }
}

/**
 * `request` compiled by `compile`, or what `cache` kept for it: a hit doesn't compile, a miss compiles and keeps the
 * output. A cache that fails, or a page without `crypto.subtle`, only compiles.
 *
 * @throws Error when the source doesn't compile, as `compile` throws.
 */
export async function compiledWith(cache: CompiledCache | null, request: CompileRequest, compile: (request: CompileRequest) => Promise<Compiled>): Promise<Compiled> {
  const key = cache ? await compiledKey(request).catch(() => undefined) : undefined;
  if (!cache || key === undefined) return compile(request);
  const kept = await cache.get(key).catch(() => undefined);
  if (kept) return kept;
  const compiled = await compile(request);
  await cache.set(key, compiled).catch(() => undefined);
  return compiled;
}
