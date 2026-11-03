// `host` (spec §7.4): what a component given it can call and hear, each use a test, the same across a frame and in the
// host's page. `serveHost` is the host's page's end of a permissive frame's `host`, `hostClient` what the component in
// the frame gets; `localHost` is what a trusted component in the host's page gets.

import { afterEach, describe, expect, it, vi } from "vitest";
import { hostClient, localHost, serveHost, type HostApi, type HostClient } from "./host-api.ts";

const ports: MessagePort[] = [];
afterEach(() => {
  for (const port of ports.splice(0)) port.close();
});

/** The host's `api` served on one end of a channel, and a permissive component's client on the other. */
function acrossAFrame(api: HostApi): { emit: (event: string, data: unknown) => void; host: HostClient } {
  const { port1, port2 } = new MessageChannel();
  ports.push(port1, port2);
  return { emit: serveHost(port1, api), host: hostClient(port2) };
}

/** The host's `api` as a trusted component in the host's page gets it. */
function inThePage(api: HostApi): { emit: (event: string, data: unknown) => void; host: HostClient } {
  const { client, emit } = localHost(api);
  return { emit, host: client };
}

/** Lets the events posted so far arrive. */
const settle = () => new Promise((resolve) => setTimeout(resolve, 10));

const WHERE = [
  ["across a frame", acrossAFrame],
  ["in the host's page", inThePage],
] as const;

describe.each(WHERE)("host.call, %s", (_where, connected) => {
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

describe.each(WHERE)("host.on, %s", (_where, connected) => {
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
