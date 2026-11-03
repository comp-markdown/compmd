// The app's server, without Vite: the page and its documents, the sandbox's frames, the packs' built modules and the
// registry proxy. `node server.ts public 8080 compmd.config.json`
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { PackRelease } from "@compmd/processor";
import { NpmRegistry, registryProxy } from "@compmd/registry";
import { toNodeListener } from "@compmd/registry/node";
import { libraryDefinition } from "@compmd/sandbox/build";
import { policyOf, readConfig } from "@compmd/vite/config";

const [root = "public", port = "8080", configFile = "compmd.config.json"] = process.argv.slice(2);

// The configuration, checked as the Vite plugin checks it, and translated for the processor, with what the frames have.
const config = readConfig(JSON.parse(await readFile(configFile, "utf8")));
const { policy, ambient } = policyOf(config, (key) => key);
const { runtimes, libraries } = await libraryDefinition(join(root, "compmd"));
const named = ({ name, version }: { name: string; version: string }) => ({ name, version });

// An installed pack, given by hand: its manifest, its document, and its built module as empty text, which the frames load.
const pack = dirname(fileURLToPath(import.meta.resolve("@comp-markdown/mermaid/package.json")));
const manifest = await readFile(join(pack, "package.json"), "utf8");
const { name, version } = JSON.parse(manifest) as { name: string; version: string };
const files = { "package.json": manifest, "index.comp.md": await readFile(join(pack, "index.comp.md"), "utf8"), "dist/mermaid.js": "" };
const release: PackRelease = { version, integrity: null, files };
const place = `${name}@${version}/dist/mermaid.js`;
const module = await readFile(join(pack, "dist/mermaid.js"));

// What the page needs to open a document: the request's packs and policy, and each built module's hash.
const app = JSON.stringify({
  request: { packs: { [name]: [release] }, ambient, policy: { ...policy, runtimes: runtimes.map(named), libraries: libraries.map(named) } },
  modules: { [place]: `sha384-${createHash("sha384").update(module).digest("base64")}` },
});

// The registries' proxy, when the configuration names registries and the host fetches from them.
const registries = Object.fromEntries(Object.entries(config.registries).map(([scope, url]) => [scope, new NpmRegistry({ url })]));
const proxy = config.fetch === "host" && Object.keys(registries).length > 0 ? toNodeListener(registryProxy({ registries, base: "/compmd/registry/" })) : undefined;

const TYPES: Record<string, string> = { ".html": "text/html", ".js": "text/javascript", ".json": "application/json", ".css": "text/css", ".wasm": "application/wasm", ".md": "text/markdown" };

/** The frames have an opaque origin, so their files load cross-origin; their documents are sandboxed even opened alone. */
const framed = (path: string): Record<string, string> => ({ "access-control-allow-origin": "*", ...(path.endsWith(".html") ? { "content-security-policy": "sandbox allow-scripts" } : {}) });

function send(response: ServerResponse, path: string, body: string | Buffer, headers: Record<string, string> = {}): void {
  response.writeHead(200, { "content-type": TYPES[extname(path)] ?? "application/octet-stream", ...headers }).end(body);
}

createServer((request, response) => {
  // The URL's path, its `..` segments resolved, so a file is always under `root`.
  const path = new URL(request.url ?? "/", "http://localhost").pathname;
  if (proxy && path.startsWith("/compmd/registry/")) return proxy(request, response);
  if (path === "/app.json") return send(response, path, app);
  if (path === `/compmd/packs/${place}`) return send(response, path, module, framed(path));
  const file = join(root, path === "/" ? "index.html" : path);
  void readFile(file).then(
    (body) => send(response, file, body, path.startsWith("/compmd/") ? framed(path) : {}),
    () => response.writeHead(404).end(),
  );
}).listen(Number(port), () => console.log(`http://localhost:${port}/`));
