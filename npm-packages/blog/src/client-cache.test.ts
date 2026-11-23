import { describe, expect, it, vi } from "vitest";
import { ClientCache } from "./client-cache.ts";
import type { Client } from "./live.ts";

const client = (script: string): Client => ({ files: new Map(), script });

describe("ClientCache", () => {
  it("builds the script once, and again once it is forgotten", async () => {
    const clients = new ClientCache();
    const build = vi.fn().mockResolvedValueOnce(client("one.js")).mockResolvedValueOnce(client("two.js"));
    expect((await clients.get(build)).script).toBe("one.js");
    expect((await clients.get(build)).script).toBe("one.js");
    clients.forget();
    expect((await clients.get(build)).script).toBe("two.js");
    expect(build).toHaveBeenCalledTimes(2);
  });

  it("doesn't keep a build that failed, so the next one tries again", async () => {
    const clients = new ClientCache();
    await expect(clients.get(() => Promise.reject(new Error("no")))).rejects.toThrow("no");
    expect((await clients.get(() => Promise.resolve(client("fixed.js")))).script).toBe("fixed.js");
  });

  it("keeps a build made after a forget, when the one before it fails late", async () => {
    const clients = new ClientCache();
    let fail: (error: Error) => void = () => undefined;
    const first = clients.get(() => new Promise((_resolve, reject) => (fail = reject)));
    clients.forget();
    const second = clients.get(() => Promise.resolve(client("second.js")));
    fail(new Error("late"));
    await expect(first).rejects.toThrow("late");
    expect(await clients.get(() => Promise.resolve(client("third.js")))).toBe(await second);
  });
});
