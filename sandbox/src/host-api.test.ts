// `host` across a frame (spec §7.4): what a permissive component of a pack given `host` can call and hear, each use a
// test. `serveHost` is the host's page's end; `hostClient` is what the component gets.

import { afterEach, describe, expect, it, vi } from "vitest";
import { hostClient, serveHost } from "./host-api.ts";

const ports: MessagePort[] = [];
afterEach(() => {
  for (const port of ports.splice(0)) port.close();
});

/** The host's `api` served on one end of a channel, and the component's client on the other. */
function connected(api: Parameters<typeof serveHost>[1]) {
  const { port1, port2 } = new MessageChannel();
  ports.push(port1, port2);
  return { emit: serveHost(port1, api), host: hostClient(port2) };
}

/** Lets the events posted so far arrive. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 10));

describe("host.call", () => {
  it("calls the host's function by name with the arguments given, and gives what it returns", async () => {
    const { host } = connected({ rating: (product) => ({ product, stars: 4 }) });
    await expect(host.call("rating", "p-1")).resolves.toEqual({ product: "p-1", stars: 4 });
  });

  it("waits for a function that returns a promise", async () => {
    const { host } = connected({ later: () => Promise.resolve("done") });
    await expect(host.call("later")).resolves.toBe("done");
  });

  it("fails with the host's message when its function throws", async () => {
    const { host } = connected({
      broken: () => {
        throw new Error("no access");
      },
    });
    await expect(host.call("broken")).rejects.toThrow("no access");
  });

  it("fails when its function throws what isn't an Error", async () => {
    const { host } = connected({
      odd: () => {
        // eslint-disable-next-line @typescript-eslint/only-throw-error -- a host's function may throw anything
        throw "no";
      },
    });
    await expect(host.call("odd")).rejects.toThrow("no");
  });

  it("fails for a name the host doesn't give, even one every object has", async () => {
    const { host } = connected({});
    await expect(host.call("missing")).rejects.toThrow("the host has no `missing`");
    await expect(host.call("toString")).rejects.toThrow("the host has no `toString`");
  });
});

describe("host.on", () => {
  it("hears the host's events until it stops listening", async () => {
    const { emit, host } = connected({});
    const heard: unknown[] = [];
    const stop = host.on("changed", (data) => heard.push(data));
    emit("changed", { stars: 5 });
    // A busy machine may take longer than `settle` to deliver it.
    await vi.waitFor(() => {
      expect(heard).toHaveLength(1);
    });
    stop();
    emit("changed", { stars: 1 });
    await settle();
    expect(heard).toEqual([{ stars: 5 }]);
  });

  it("tells every listener of an event, and only those of that event", async () => {
    const { emit, host } = connected({});
    const heard: string[] = [];
    host.on("changed", () => heard.push("first"));
    host.on("changed", () => heard.push("second"));
    host.on("removed", () => heard.push("removed"));
    emit("changed", {});
    await settle();
    expect(heard).toEqual(["first", "second"]);
  });
});
