// An app with registries (docs/vite.md, "Packs from a registry"), on the dev server and in a build served by
// `vite preview`: with `fetch: "host"`, the page's registries are the app's proxy, which passes requests on to the
// registries the configuration names; with `direct`, they are those registries, and there is no proxy.
import { mkdir, rm, writeFile } from "node:fs/promises";
import { createServer as createHttpServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { dirname, join } from "node:path";
import { chromium, type Browser } from "playwright";
import { build, createServer, preview, type InlineConfig } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "../src/plugin.ts";

const APP = join(import.meta.dirname, ".out/registry-app");
const PORT = 4198;
const TARBALL = Buffer.from("the tarball's bytes");

let upstream: Server;
let upstreamUrl = "";
let browser: Browser;

/** A registry that has `@kit/bars@1.0.0`, its tarball at `tarballs/`. */
async function startUpstream(): Promise<void> {
  upstream = createHttpServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    if (path === "/tarballs/bars-1.0.0.tgz") return response.writeHead(200, { "content-type": "application/octet-stream" }).end(TARBALL);
    if (path !== "/@kit/bars") return response.writeHead(404).end();
    const dist = { tarball: `${upstreamUrl}tarballs/bars-1.0.0.tgz`, integrity: "sha512-bars", unpackedSize: 2000 };
    response.writeHead(200, { "content-type": "application/vnd.npm.install-v1+json", "cache-control": "max-age=300" });
    response.end(JSON.stringify({ name: "@kit/bars", versions: { "1.0.0": { version: "1.0.0", dist } } }));
  });
  await new Promise<void>((resolve) => upstream.listen(0, "127.0.0.1", resolve));
  upstreamUrl = `http://127.0.0.1:${String((upstream.address() as AddressInfo).port)}/`;
}

/** Writes the app, with `config` as its `compmd.config.json`; its page puts `virtual:compmd`'s `registry` on `window`. */
async function writeApp(config: object): Promise<void> {
  const files: Record<string, string> = {
    "package.json": JSON.stringify({ name: "registry-app", private: true, type: "module" }),
    "compmd.config.json": JSON.stringify(config),
    "index.html": '<!doctype html><title>Registry app</title><script type="module" src="/main.js"></script>',
    "main.js": 'import { registry } from "virtual:compmd";\nwindow.registry = registry;\n',
  };
  await rm(APP, { recursive: true, force: true });
  for (const [path, content] of Object.entries(files)) {
    await mkdir(dirname(join(APP, path)), { recursive: true });
    await writeFile(join(APP, path), content);
  }
}

const viteConfig = (port: number): InlineConfig => ({
  root: APP,
  configFile: false,
  logLevel: "warn",
  plugins: [compmd()],
  build: { outDir: join(APP, "dist"), emptyOutDir: true, target: "es2024" },
  server: { port, strictPort: true },
  preview: { port, strictPort: true },
});

/** What the page at `url` gets from its `registry`: who fetches, the cap, and where `@kit/bars`'s tarball is. */
async function inPage(url: string): Promise<{ fetch: string; maxPackSize?: number; tarball: string | undefined }> {
  const page = await browser.newPage();
  try {
    await page.goto(url);
    await page.waitForFunction(() => "registry" in window);
    return await page.evaluate(async () => {
      const { registry } = window as unknown as { registry: { fetch: string; maxPackSize?: number; registries: Record<string, { versions(name: string): Promise<{ versions: { tarball: string }[] }> }> } };
      const { versions } = await (registry.registries["@kit"] ?? registry.registries["*"])!.versions("@kit/bars");
      return { fetch: registry.fetch, ...(registry.maxPackSize === undefined ? {} : { maxPackSize: registry.maxPackSize }), tarball: versions[0]?.tarball };
    });
  } finally {
    await page.close();
  }
}

beforeAll(async () => {
  await startUpstream();
  browser = await chromium.launch({ headless: true, channel: "chrome" });
});

afterAll(async () => {
  await browser.close();
  upstream.closeAllConnections();
  await new Promise((resolve) => upstream.close(resolve));
  await rm(APP, { recursive: true, force: true });
});

describe("registries with fetch: host", () => {
  it("makes the dev server the app's proxy, and the page's registries that proxy", async () => {
    await writeApp({ registries: { "*": upstreamUrl }, maxPackSize: "2 MB" });
    const dev = await createServer(viteConfig(PORT));
    await dev.listen();
    const origin = `http://localhost:${String(PORT)}`;
    try {
      const metadata = await fetch(`${origin}/compmd/registry/@kit%2fbars`);
      expect(metadata.status).toBe(200);
      const { versions } = (await metadata.json()) as { versions: Record<string, { dist: { tarball: string } }> };
      expect(versions["1.0.0"]?.dist.tarball).toBe(`${origin}/compmd/registry/-/tarball/@kit/bars/1.0.0`);
      expect(Buffer.from(await (await fetch(`${origin}/compmd/registry/-/tarball/@kit/bars/1.0.0`)).arrayBuffer())).toEqual(TARBALL);
      expect(await inPage(`${origin}/`)).toEqual({ fetch: "host", maxPackSize: 2_000_000, tarball: `${origin}/compmd/registry/-/tarball/@kit/bars/1.0.0` });
    } finally {
      await dev.close();
    }
  }, 60_000);

  it("makes `vite preview` the proxy of the built app", async () => {
    await writeApp({ registries: { "@kit": upstreamUrl } });
    await build(viteConfig(PORT));
    const server = await preview(viteConfig(PORT));
    const origin = `http://localhost:${String(PORT)}`;
    try {
      expect(await inPage(`${origin}/`)).toEqual({ fetch: "host", tarball: `${origin}/compmd/registry/-/tarball/@kit/bars/1.0.0` });
    } finally {
      await server.close();
    }
  }, 60_000);
});

describe("registries with fetch: direct", () => {
  it("gives the page the registries themselves, and serves no proxy", async () => {
    await writeApp({ registries: { "@kit": upstreamUrl }, fetch: "direct" });
    const dev = await createServer(viteConfig(PORT));
    await dev.listen();
    const origin = `http://localhost:${String(PORT)}`;
    try {
      expect((await fetch(`${origin}/compmd/registry/@kit%2fbars`)).headers.get("content-type")).not.toContain("npm");
      // The upstream answers no CORS request: the page reads its registry's URL, not its metadata.
      const { registry } = (await dev.ssrLoadModule("virtual:compmd")) as { registry: { fetch: string; registries: Record<string, { url: string }> } };
      expect(registry.fetch).toBe("direct");
      expect(registry.registries["@kit"]?.url).toBe(upstreamUrl);
    } finally {
      await dev.close();
    }
  }, 60_000);
});

describe("no registries", () => {
  it("gives no registry, and serves no proxy", async () => {
    await writeApp({});
    const dev = await createServer(viteConfig(PORT));
    await dev.listen();
    try {
      const { registry } = (await dev.ssrLoadModule("virtual:compmd")) as { registry: unknown };
      expect(registry).toBeUndefined();
      expect((await fetch(`http://localhost:${String(PORT)}/compmd/registry/@kit%2fbars`)).headers.get("content-type")).not.toContain("npm");
    } finally {
      await dev.close();
    }
  }, 60_000);
});
