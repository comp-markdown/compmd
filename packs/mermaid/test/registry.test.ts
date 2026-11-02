// The pack as a host gets it from a registry (docs/mermaid-pack.md, Tests): its tarball as `pnpm pack` writes it,
// served by a local registry; a host that installs no pack resolves the document's import through its proxy, checks
// the tarball, and gives the frame the built module as code.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { mkdir, rm } from "node:fs/promises";
import { createServer as createHttpServer, type Server } from "node:http";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";
import { chromium, type Browser } from "playwright";
import { createServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "@compmd/vite";

const ROOT = join(import.meta.dirname, "..");
const OUT = join(import.meta.dirname, ".out/registry");
const APP = join(import.meta.dirname, "registry-app");
/** The registry's port, which the app's configuration names. */
const REGISTRY_PORT = 4231;
const APP_PORT = 4232;
const NAME = "@comp-markdown/mermaid";

let tarball: Buffer;
let manifest: { version: string };
let registry: Server;
let app: ViteDevServer;
let browser: Browser;
const requests: string[] = [];

/** The sum of the sizes of the files in a gzipped tar: the unpacked size npm's metadata states. */
function unpackedSize(gzipped: Buffer): number {
  const tar = gunzipSync(gzipped);
  let total = 0;
  for (let at = 0; at + 512 <= tar.length; ) {
    const size = parseInt(tar.subarray(at + 124, at + 136).toString("ascii").replace(/\0.*$/, "").trim() || "0", 8);
    if (tar[at] === 0) break;
    if (tar[at + 156] === 0x30 || tar[at + 156] === 0) total += size;
    at += 512 + Math.ceil(size / 512) * 512;
  }
  return total;
}

beforeAll(async () => {
  await rm(OUT, { recursive: true, force: true });
  await mkdir(OUT, { recursive: true });
  execFileSync("pnpm", ["pack", "--pack-destination", OUT], { cwd: ROOT, stdio: "pipe" });
  const file = readdirSync(OUT).find((name) => name.endsWith(".tgz")) ?? "";
  tarball = readFileSync(join(OUT, file));
  manifest = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8")) as { version: string };
  const integrity = `sha512-${createHash("sha512").update(tarball).digest("base64")}`;
  registry = createHttpServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    requests.push(path);
    if (path === "/tarball.tgz") return response.writeHead(200, { "content-type": "application/octet-stream" }).end(tarball);
    if (path !== `/${NAME}`) return response.writeHead(404).end();
    const dist = { tarball: `http://127.0.0.1:${String(REGISTRY_PORT)}/tarball.tgz`, integrity, unpackedSize: unpackedSize(tarball) };
    response.writeHead(200, { "content-type": "application/vnd.npm.install-v1+json" });
    response.end(JSON.stringify({ name: NAME, versions: { [manifest.version]: { version: manifest.version, dist } } }));
  });
  await new Promise<void>((resolve) => registry.listen(REGISTRY_PORT, "127.0.0.1", resolve));
  app = await createServer({ root: APP, configFile: false, logLevel: "warn", plugins: [compmd()], server: { port: APP_PORT, strictPort: true } });
  await app.listen();
  browser = await chromium.launch({ headless: true, channel: "chrome" });
}, 120_000);

afterAll(async () => {
  await browser.close();
  await app.close();
  registry.closeAllConnections();
  await new Promise((resolve) => registry.close(resolve));
});

describe("the pack from a registry", () => {
  // That it is one module, under the default size cap, is `compmd-pack build`'s check.
  it("publishes the licences of what it bundles", () => {
    expect(readdirSync(join(ROOT, "dist"))).toContain("LICENSES.md");
    expect(readFileSync(join(ROOT, "dist/LICENSES.md"), "utf8")).toContain("mermaid");
  });

  it("draws in a host that installs no pack: fetched through its proxy, checked, and given to the frame as code", async () => {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`http://localhost:${String(APP_PORT)}/`);
    const drawn = page.frameLocator("iframe").locator('[role="img"][aria-label="Fetched, then drawn"] svg');
    await expect.poll(() => drawn.count(), { timeout: 30_000 }).toBe(1);
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
    // The proxy reads the metadata again to find the tarball; the tarball is fetched once.
    expect([...new Set(requests)]).toEqual([`/${NAME}`, "/tarball.tgz"]);
    expect(requests.filter((path) => path === "/tarball.tgz")).toHaveLength(1);
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);
});
