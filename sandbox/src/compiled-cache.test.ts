import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { cacheStorage, COMPILER, compiledKey, compiledWith, type Compiled, type CompiledCache } from "./compiled-cache.ts";
import type { CompileRequest } from "./protocol/messages.ts";

const request: CompileRequest = { name: "chart", language: "jsx", code: "export default () => <svg />;" };

/** A cache in memory, which counts what it is asked. */
function memory(): CompiledCache & { kept: Map<string, Compiled>; gets: number } {
  const kept = new Map<string, Compiled>();
  return {
    kept,
    gets: 0,
    get(key) {
      this.gets += 1;
      return Promise.resolve(kept.get(key));
    },
    set: (key, compiled) => {
      kept.set(key, compiled);
      return Promise.resolve();
    },
  };
}

/** A compiler that counts its calls. */
function compiler() {
  const calls: CompileRequest[] = [];
  const compile = (asked: CompileRequest): Promise<Compiled> => {
    calls.push(asked);
    return Promise.resolve({ code: `compiled ${asked.code}` });
  };
  return { calls, compile };
}

describe("compiledKey", () => {
  it("is SHA-256 over the language, the runtime, the compiler and the source, not the name", async () => {
    const key = await compiledKey(request);
    expect(key).toMatch(/^[0-9a-f]{64}$/);
    const renamed: CompileRequest = { ...request, name: "other" };
    expect(await compiledKey(renamed)).toBe(key);
    expect(await compiledKey({ ...request, language: "tsx" })).not.toBe(key);
    expect(await compiledKey({ ...request, runtime: "react@19" })).not.toBe(key);
    expect(await compiledKey({ ...request, code: `${request.code} ` })).not.toBe(key);
  });

  it("names the compilers the package depends on, so an upgrade misses what was kept", () => {
    const { dependencies } = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8")) as { dependencies: Record<string, string> };
    expect(COMPILER).toContain(`sucrase@${dependencies.sucrase ?? ""}`);
    expect(COMPILER).toContain(`svelte@${dependencies.svelte ?? ""}`);
  });
});

describe("compiledWith", () => {
  it("compiles a miss and keeps it, and gives a hit without compiling", async () => {
    const cache = memory();
    const { calls, compile } = compiler();
    expect(await compiledWith(cache, request, compile)).toEqual({ code: `compiled ${request.code}` });
    expect(await compiledWith(cache, { ...request, name: "again" }, compile)).toEqual({ code: `compiled ${request.code}` });
    expect(calls).toHaveLength(1);
    expect(cache.kept.size).toBe(1);
  });

  it("only compiles without a cache, or with one that throws", async () => {
    const { calls, compile } = compiler();
    await compiledWith(null, request, compile);
    const broken: CompiledCache = { get: () => Promise.reject(new Error("gone")), set: () => Promise.reject(new Error("gone")) };
    expect(await compiledWith(broken, request, compile)).toEqual({ code: `compiled ${request.code}` });
    expect(calls).toHaveLength(2);
  });

  it("keeps nothing when the source doesn't compile", async () => {
    const cache = memory();
    await expect(compiledWith(cache, request, () => Promise.reject(new Error("unexpected token")))).rejects.toThrow("unexpected token");
    expect(cache.kept.size).toBe(0);
  });
});

describe("cacheStorage", () => {
  /** Cache Storage in memory, as a browser's: entries by URL, as responses. */
  function storage(): CacheStorage {
    const entries = new Map<string, Response>();
    const cache = {
      match: (url: string) => Promise.resolve(entries.get(url)?.clone()),
      put: (url: string, response: Response) => {
        entries.set(url, response);
        return Promise.resolve();
      },
    };
    return { open: () => Promise.resolve(cache) } as unknown as CacheStorage;
  }

  it("keeps compiled code as JSON, under a URL that is never fetched", async () => {
    const cache = cacheStorage(storage());
    await cache.set("abc", { code: "x", css: "y" });
    expect(await cache.get("abc")).toEqual({ code: "x", css: "y" });
    expect(await cache.get("other")).toBeUndefined();
  });

  it("misses, and keeps nothing, where Cache Storage is missing or refuses", async () => {
    const refusing = { open: () => Promise.reject(new Error("SecurityError")) } as unknown as CacheStorage;
    for (const cache of [cacheStorage(undefined), cacheStorage(refusing)]) {
      await cache.set("abc", { code: "x" });
      expect(await cache.get("abc")).toBeUndefined();
    }
  });
});
