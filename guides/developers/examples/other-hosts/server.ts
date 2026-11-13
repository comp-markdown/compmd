// The app's server, without Vite: the page and its documents, the bundle under `compmd/` as the serving contract says,
// and the registry proxy. `node server.ts public bundle 8080 compmd.config.json`
import { readFile } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";
import { extname, join } from "node:path";
import type { PackRelease } from "@compmd/processor";
import { NpmRegistry, registryProxy } from "@compmd/registry";
import { toNodeListener } from "@compmd/registry/node";
import { CONTRACT_HEADERS } from "@compmd/sandbox/check";
import type { BundledPack, Manifest } from "@compmd/vite";
import { readConfig } from "@compmd/vite/config";

const [root = "public", bundle = "bundle", port = "8080", configFile = "compmd.config.json"] = process.argv.slice(2);

// The bundle's one description: the frames' hash, its packs with their modules' hashes, and the processor's policy.
const manifest = JSON.parse(await readFile(join(bundle, "manifest.json"), "utf8")) as Manifest;

/** A pack of the bundle as the processor takes it: its manifest, its document, and its built modules as empty text. */
async function releaseOf({ pack, version, integrity, modules }: BundledPack): Promise<PackRelease> {
  const folder = join(bundle, "packs", `${pack}@${version}`);
  const packageJson = await readFile(join(folder, "package.json"), "utf8");
  const named = (JSON.parse(packageJson) as { compmd?: unknown }).compmd;
  const document = (typeof named === "string" ? named : "index.comp.md").replace(/^\.\//, "");
  const built = Object.fromEntries(Object.keys(modules).map((path) => [path, ""]));
  return { version, integrity: integrity ?? null, files: { ...built, "package.json": packageJson, [document]: await readFile(join(folder, document), "utf8") } };
}

// What the page needs to open a document: the request's packs, policy and ambient packs, and where the bundle's frames
// and modules are under its origin, each module with its file's hash.
const packs: Record<string, PackRelease[]> = {};
for (const bundled of manifest.packs) (packs[bundled.pack] ??= []).push(await releaseOf(bundled));
const modules = Object.fromEntries(
  manifest.packs.flatMap(({ pack, version, modules }) => Object.entries(modules).map(([path, integrity]) => [`${pack}@${version}/${path}`, { path: `packs/${pack}@${version}/${path}`, integrity }])),
);
const app = JSON.stringify({ request: { packs, policy: manifest.policy, ambient: manifest.ambient }, frames: `frames/${manifest.frames.hash}/`, modules });

// The registries' proxy, at `registry/` under the bundle's origin, when the configuration names registries and the host
// fetches from them.
const config = readConfig(JSON.parse(await readFile(configFile, "utf8")));
const registries = Object.fromEntries(Object.entries(config.registries).map(([scope, url]) => [scope, new NpmRegistry({ url })]));
const proxy = config.fetch === "host" && Object.keys(registries).length > 0 ? toNodeListener(registryProxy({ registries, base: "/compmd/registry/" })) : undefined;

const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".wasm": "application/wasm", ".md": "text/markdown" };

function send(response: ServerResponse, status: number, path: string, body: string | Buffer, headers: Record<string, string> = {}): void {
  response.writeHead(status, { "content-type": TYPES[extname(path)] ?? "application/octet-stream", ...headers }).end(body);
}

/**
 * The serving contract: a file of the frames' build, `frames/<hash>/<path>`, or of a bundled pack, `packs/<place>`, with
 * the contract's headers. Anything else, the manifest too, is a 404 a frame's page can read.
 */
function serveBundle(response: ServerResponse, path: string): void {
  const refuse = (why: string) => send(response, 404, ".txt", why, { "access-control-allow-origin": "*" });
  if (path.split("/").some((part) => part === "" || part === "." || part === "..")) return refuse("not a file of the bundle");
  const ours = path.startsWith(`frames/${manifest.frames.hash}/`) || manifest.packs.some(({ pack, version }) => path.startsWith(`packs/${pack}@${version}/`));
  if (!ours) return refuse("not a file of the bundle");
  void readFile(join(bundle, path)).then(
    (body) => send(response, 200, path, body, CONTRACT_HEADERS),
    () => refuse("no such file in the bundle"),
  );
}

createServer((request, response) => {
  // The URL's path, its `..` segments resolved, so a file is always under `root`.
  const path = new URL(request.url ?? "/", "http://localhost").pathname;
  if (proxy && path.startsWith("/compmd/registry/")) return proxy(request, response);
  if (path.startsWith("/compmd/")) return serveBundle(response, decodeURIComponent(path.slice("/compmd/".length)));
  if (path === "/app.json") return send(response, 200, path, app);
  const file = join(root, path === "/" ? "index.html" : path);
  void readFile(file).then(
    (body) => send(response, 200, file, body),
    () => response.writeHead(404).end(),
  );
}).listen(Number(port), () => console.log(`http://localhost:${port}/`));
