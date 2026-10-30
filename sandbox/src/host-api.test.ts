import { describe, expect, it } from "vitest";
import { hostClient, serveHost } from "./host-api.ts";

describe("host across a frame", () => {
  it("calls the host's functions by name, and hears its events", async () => {
    const { port1, port2 } = new MessageChannel();
    const emit = serveHost(port1, {
      rating: (product) => ({ product, stars: 4 }),
      later: async () => Promise.resolve("done"),
      broken: () => {
        throw new Error("no access");
      },
    });
    const host = hostClient(port2);
    await expect(host.call("rating", "p-1")).resolves.toEqual({ product: "p-1", stars: 4 });
    await expect(host.call("later")).resolves.toBe("done");
    await expect(host.call("broken")).rejects.toThrow("no access");
    await expect(host.call("missing")).rejects.toThrow("the host has no `missing`");
    await expect(host.call("toString")).rejects.toThrow("the host has no `toString`");
    const heard: unknown[] = [];
    const stop = host.on("changed", (data) => heard.push(data));
    emit("changed", { stars: 5 });
    await new Promise((resolve) => setTimeout(resolve, 10));
    stop();
    emit("changed", { stars: 1 });
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(heard).toEqual([{ stars: 5 }]);
    port1.close();
  });
});
