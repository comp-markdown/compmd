// `registryProxy` and `toNodeListener`, as a host serves them: one scenario a test, each a good use or a bad one. The
// packs are the local registry's (test/local-registry.ts).

import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { localRegistry, tarballOf, type LocalRegistry } from "../test/local-registry.ts";
import { toNodeListener } from "./node.ts";
import { registryProxy } from "./proxy.ts";
import { NpmRegistry } from "./registry.ts";

let local: LocalRegistry;
let proxy: (request: Request) => Promise<Response>;
beforeAll(async () => {
  local = await localRegistry();
  proxy = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url }) }, base: "/compmd/registry/" });
});
afterAll(() => local.close());

/** A request to the proxy, served at `https://notes.example/compmd/registry/`. */
const get = (path: string, init?: RequestInit) => proxy(new Request(`https://notes.example${path}`, init));

/** Runs `use` while the local registry answers 500 to everything. */
async function whileFailing<T>(use: () => Promise<T>): Promise<T> {
  local.failing(true);
  try {
    return await use();
  } finally {
    local.failing(false);
  }
}

describe("registryProxy: what it answers", () => {
  it("passes a pack's metadata on from its registry, each tarball's URL its own, with the registry's max-age", async () => {
    const response = await get("/compmd/registry/@kit%2fcharts");
    expect(response.status).toBe(200);
    expect(response.headers.get("cache-control")).toBe("public, max-age=300");
    const { versions } = (await response.json()) as { versions: Record<string, { dist: { tarball: string } }> };
    expect(versions["1.1.0"]?.dist.tarball).toBe("https://notes.example/compmd/registry/-/tarball/@kit/charts/1.1.0");
  });

  it("streams a tarball from where its registry says it is, to be kept for good: a version never changes", async () => {
    const response = await get("/compmd/registry/-/tarball/@kit/charts/1.0.0");
    expect(response.headers.get("cache-control")).toBe("public, max-age=31536000, immutable");
    expect(Buffer.from(await response.arrayBuffer())).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
  });

  it("is served at `/` when no base is given", async () => {
    const atRoot = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url }) } });
    expect((await atRoot(new Request("https://notes.example/@kit%2fcharts"))).status).toBe(200);
  });

  it("asks the registry of a pack's scope", async () => {
    const scoped = registryProxy({ registries: { "@kit": new NpmRegistry({ url: local.url }) } });
    expect((await scoped(new Request("https://notes.example/@kit%2fcharts"))).status).toBe(200);
  });

  it("sends the registry its own headers, such as a token, and nothing the reader sent", async () => {
    const upstream: Headers[] = [];
    const recording: typeof fetch = (url, init) => {
      upstream.push(new Headers(init?.headers));
      return fetch(url, init);
    };
    const withToken = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url, fetch: recording, headers: { authorization: "Bearer registry" } }) } });
    await withToken(new Request("https://notes.example/@kit%2fcharts", { headers: { cookie: "session=reader", authorization: "Bearer reader" } }));
    expect(upstream.map((headers) => [headers.get("authorization"), headers.get("cookie")])).toEqual([["Bearer registry", null]]);
  });
});

describe("registryProxy: what it refuses (404)", () => {
  it.each([
    ["a method but GET", "/compmd/registry/@kit%2fcharts", { method: "POST" }],
    ["a path outside its base", "/elsewhere/@kit%2fcharts", {}],
    ["a name that isn't a pack's", "/compmd/registry/charts", {}],
    ["a pack its registry doesn't have", "/compmd/registry/@kit%2fnothing", {}],
    ["a version its registry doesn't have", "/compmd/registry/-/tarball/@kit/charts/9.9.9", {}],
    ["a search, or any other route of npm's", "/compmd/registry/-/v1/search?text=x", {}],
  ])("%s", async (_why, path, init: RequestInit) => {
    expect((await get(path, init)).status).toBe(404);
  });

  it("a pack of a scope it has no registry for, and no `*`", async () => {
    const scoped = registryProxy({ registries: { "@kit": new NpmRegistry({ url: local.url }) } });
    expect((await scoped(new Request("https://notes.example/@other%2fpack"))).status).toBe(404);
  });
});

describe("registryProxy: when its registry fails", () => {
  it("is a bad gateway (502) for metadata", async () => {
    expect((await whileFailing(() => get("/compmd/registry/@kit%2fcharts"))).status).toBe(502);
  });

  it("is a bad gateway (502) for a tarball", async () => {
    expect((await whileFailing(() => get("/compmd/registry/-/tarball/@kit/charts/1.0.0"))).status).toBe(502);
  });
});

describe("toNodeListener", () => {
  /** Serves `handler` with Node's http on a free port while `use` runs; gives `use` the server's URL. */
  async function serving(handler: (request: Request) => Promise<Response>, use: (url: string) => Promise<void>): Promise<void> {
    const server: Server = createServer(toNodeListener(handler));
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    try {
      await use(`http://127.0.0.1:${String((server.address() as AddressInfo).port)}`);
    } finally {
      server.closeAllConnections();
      server.close();
    }
  }

  it("serves the proxy through Node's http, streaming its body", async () => {
    await serving(
      (request) => get(new URL(request.url).pathname),
      async (url) => {
        const response = await fetch(`${url}/compmd/registry/-/tarball/@kit/charts/1.0.0`);
        expect(Buffer.from(await response.arrayBuffer())).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
      },
    );
  });

  it("passes the handler none of the reader's cookies or credentials", async () => {
    let seen: Request | undefined;
    await serving(
      (request) => {
        seen = request;
        return Promise.resolve(new Response("ok"));
      },
      async (url) => {
        await fetch(`${url}/`, { headers: { cookie: "session=secret", authorization: "Bearer reader" } });
      },
    );
    expect(seen?.headers.get("cookie")).toBeNull();
    expect(seen?.headers.get("authorization")).toBeNull();
  });

  it("answers 500 when the handler fails", async () => {
    await serving(
      () => Promise.reject(new Error("broken")),
      async (url) => {
        expect((await fetch(`${url}/`)).status).toBe(500);
      },
    );
  });

  it("answers a response with no body as it is", async () => {
    await serving(
      () => Promise.resolve(new Response(null, { status: 204 })),
      async (url) => {
        expect((await fetch(`${url}/`)).status).toBe(204);
      },
    );
  });
});
