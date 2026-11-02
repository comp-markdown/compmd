import { createServer } from "node:http";
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

const get = (path: string, init?: RequestInit) => proxy(new Request(`https://notes.example${path}`, init));

describe("registryProxy", () => {
  it("passes a pack's metadata on, each tarball's URL its own", async () => {
    const response = await get("/compmd/registry/@kit%2fcharts");
    expect(response.headers.get("cache-control")).toBe("public, max-age=300");
    const { versions } = (await response.json()) as { versions: Record<string, { dist: { tarball: string } }> };
    expect(versions["1.1.0"]?.dist.tarball).toBe("https://notes.example/compmd/registry/-/tarball/@kit/charts/1.1.0");
  });

  it("streams a tarball from where its registry says it is", async () => {
    const response = await get("/compmd/registry/-/tarball/@kit/charts/1.0.0");
    expect(response.headers.get("cache-control")).toContain("immutable");
    expect(Buffer.from(await response.arrayBuffer())).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
  });

  it("answers nothing else", async () => {
    const statuses = await Promise.all(
      [
        get("/compmd/registry/@kit%2fcharts", { method: "POST" }),
        get("/elsewhere/@kit%2fcharts"),
        get("/compmd/registry/charts"),
        get("/compmd/registry/@kit%2fnothing"),
        get("/compmd/registry/-/tarball/@kit/charts/9.9.9"),
        get("/compmd/registry/-/v1/search?text=x"),
      ].map(async (response) => (await response).status),
    );
    expect(statuses).toEqual([404, 404, 404, 404, 404, 404]);
    const scoped = registryProxy({ registries: { "@kit": new NpmRegistry({ url: local.url }) } });
    expect((await scoped(new Request("https://notes.example/@other%2fpack"))).status).toBe(404);
  });

  it("is a bad gateway when its registry fails", async () => {
    local.failing(true);
    const response = await get("/compmd/registry/@kit%2fcharts");
    local.failing(false);
    expect(response.status).toBe(502);
  });

  it("serves through Node's http, passing on nothing the reader sent", async () => {
    let seen: Request | undefined;
    const server = createServer(
      toNodeListener((request) => {
        seen = request;
        return get(new URL(request.url).pathname);
      }),
    );
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const url = `http://127.0.0.1:${String((server.address() as AddressInfo).port)}`;
    const response = await fetch(`${url}/compmd/registry/-/tarball/@kit/charts/1.0.0`, { headers: { cookie: "session=secret" } });
    expect(Buffer.from(await response.arrayBuffer())).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
    expect(seen?.headers.get("cookie")).toBeNull();
    const failing = createServer(toNodeListener(() => Promise.reject(new Error("broken"))));
    await new Promise<void>((resolve) => failing.listen(0, "127.0.0.1", resolve));
    const failed = await fetch(`http://127.0.0.1:${String((failing.address() as AddressInfo).port)}/`);
    expect(failed.status).toBe(500);
    const empty = createServer(toNodeListener(() => Promise.resolve(new Response(null, { status: 204 }))));
    await new Promise<void>((resolve) => empty.listen(0, "127.0.0.1", resolve));
    expect((await fetch(`http://127.0.0.1:${String((empty.address() as AddressInfo).port)}/`)).status).toBe(204);
    for (const each of [server, failing, empty]) {
      each.closeAllConnections();
      each.close();
    }
  });
});
