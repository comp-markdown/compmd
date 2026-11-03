# 18. Other hosts

`@compmd/vite` does four jobs for a Vite app: it reads the configuration, gives the processor the app's packs, builds and serves the sandbox's frame documents, and serves the packs' built modules. An app built another way does them itself, with the same functions the plugin uses. This chapter builds such an app: a Node server with no Vite plugin, which serves a page, its documents, the frames, a pack's built module and the registry proxy, and a page that draws a diagram with the Mermaid pack.

## What you do by hand

| The plugin's job | Without it |
|---|---|
| reads and checks `compmd.config.json` | `readConfig` and `policyOf`, from `@compmd/vite/config` |
| builds the frame documents into `compmd/` | `buildFrames`, and `buildLibraries` for the libraries you offer, from `@compmd/sandbox/build` |
| serves `compmd/` with its two headers | your server |
| finds installed packs and gives them to the processor (`request.packs`) | `PackRelease`s you make from the packs' files |
| serves the packs' built modules, and gives the sandbox their hashes (`sandbox.modules`) | your server, and a `modules` function you write |
| serves the registry proxy | `registryProxy`, with `toNodeListener` for Node |
| builds trusted components with the app (`sandbox.trusted`) | your bundler, and the bindings of `@compmd/render` |

The example has six files: the configuration, a build script for the frames, the server, the page, its HTML, and a document.

```sh
npm install @compmd/processor @compmd/sandbox @compmd/registry @compmd/vite @comp-markdown/mermaid
```

`@compmd/vite` is here only for its configuration functions, which need no Vite to run; npm installs Vite with it all the same, as a peer dependency.

## The configuration

The same `compmd.config.json` a Vite app writes, here with one pack listed, only listed packs allowed, and npm as the registry for packs the app doesn't install:

```json snippet=other-hosts/compmd.config.json
{
  "packs": { "@comp-markdown/mermaid": {} },
  "imports": "listed",
  "registries": { "*": "https://registry.npmjs.org/" },
  "fetch": "host"
}
```

`@compmd/vite/config` reads it without Vite or Node, so it works on a server, in a worker or in a page:

- **`readConfig(value, where?)`** checks the parsed JSON and gives it with every default filled in, sizes in bytes and origins normalized (`ReadConfig`). What is wrong throws a `ConfigError`, whose `problems` lists every problem at once and whose message starts with `where` (`compmd.config.json` by default). [The configuration's reference](20-the-configurations-reference.md) has every key and every problem.
- **`policyOf(config, nameOf, refused?)`** gives what the processor takes: `policy` (the packs with their trust, `imports`, `deny`, and `refused` when you give it) and `ambient`, the names of the ambient packs. `nameOf` turns a key into a pack name: a pack name is its own, and a folder pack (`./components`) is its `package.json`'s `name`. `refused` holds the packs you leave out, each with why; the plugin puts there the packs larger than `maxPackSize`.

The module also exports `isFolder(key)`, which tells a folder key from a pack name, and the types `Config`, `ReadConfig`, `PackOptions`, `Library` and `TrustLevel`.

## The frame documents

The sandbox's frames load documents your app serves: `strict.html` and `permissive.html`, each with its content security policy and import map, the frame runtime, and the runtimes of the spec's bindings (React 19, React DOM, Svelte 5). `buildFrames` writes them, at build time, in Node:

```ts snippet=other-hosts/build.ts
// Builds the sandbox's frame documents into the folder the server serves them from: `node build.ts public/compmd`.
import { buildFrames } from "@compmd/sandbox/build";

const [out = "public/compmd"] = process.argv.slice(2);
const { runtimes } = await buildFrames({ out });
console.log(`The frames are in ${out}, with ${runtimes.map(({ name, version }) => `${name} ${version}`).join(", ")}.`);
```

It gives the frames' **library definition**, the runtimes and libraries they have, each with its version and import names, and writes it beside them as `library-definition.json`, which `libraryDefinition(out)` reads back, as the server below does.

To offer libraries besides the runtimes, as the configuration's `libraries` does, build them first, then the frames with their import map entries. The order matters: `buildFrames` copies the runtimes' files over the folder, so read the libraries before it does:

```ts
import { buildFrames, buildLibraries, readLibraries, RUNTIMES } from "@compmd/sandbox/build";

const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root, out: "public/compmd/libs", shared: RUNTIMES });
const libraries = await readLibraries("public/compmd/libs");
const definition = await buildFrames({ out: "public/compmd", imports, libraries });
```

`buildLibraries` bundles each library with Vite's bundler, so it needs Vite installed; `buildFrames` doesn't. `root` is the folder whose `node_modules` holds the libraries, and the entries it gives start with `./libs/`, so the libraries go in `libs/` beside the frame documents. The package also exports the frames' policies, `strictPolicy` and `permissivePolicy`, which `buildFrames` writes into the documents; a host never needs to write one itself.

## The server

```ts snippet=other-hosts/server.ts
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
```

What it does, part by part:

- **The policy.** `policyOf` gives the policy and the ambient packs; the server adds the frames' runtimes and libraries, by name and version, as `policy.runtimes` and `policy.libraries`. With them, the processor reports a component on a runtime the frames don't have (W102) and a pack whose `peerDependencies` they can't meet (W121). Without them, it assumes every runtime and checks no library.
- **The headers.** Every file under `compmd/` is served with `Access-Control-Allow-Origin: *`, since a frame has an opaque origin and loads its files cross-origin. The frame documents also get `Content-Security-Policy: sandbox allow-scripts`, so a frame document opened on its own, outside a frame, is sandboxed all the same. These are the headers the Vite plugin's servers send.
- **`/app.json`** is what `virtual:compmd` holds in a Vite app: the request's `packs`, `ambient` and `policy`, and each built module's hash. A server that renders many documents makes it once, as here.

## Packs, by hand

The processor never reads a disk or a registry: a pack is in the request's `packs`, by name, as a list of `PackRelease`s:

- **`version`**, the release's version, from its `package.json`.
- **`files`**: `package.json`, the pack's document (its `compmd` field names it, `index.comp.md` by default), and every file the document imports, by path in the pack. A built module (a `.js` file the document imports) can be empty text: the processor needs only to know it is there, and names it by its place in the components it gives (`module: { pack, version, path }`), for the sandbox to load. Source files (`.jsx`, `.tsx`, `.svelte`, `.ts`) need their text, which the sandbox compiles.
- **`integrity`**: the hash the registry published for the version's tarball, `sha512-…`. A document that pins its import with `integrity` gets the pack only if the two are equal (F035 otherwise). The plugin reads it from the app's lock file; this server gives `null`, so a pinned import of the pack fails.
- **`broken`**: why the files can't be used, when you found they don't match their integrity. An import of the release is then F035.

What the plugin also does with installed packs, and this server leaves out: it checks each listed pack is installed at the version its `version` range asks, leaves out packs larger than `maxPackSize` (as `refused`), and processes each listed pack's document once at build, so a broken pack stops the build. A host that wants those checks makes them; the processor still reports every problem a document meets.

## The page

The page opens a document with what the server gives, and renders it in frame mode. `modules` tells the sandbox where each built module is and its file's hash, which its frame checks before the module runs:

```ts snippet=other-hosts/page.ts
import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

await init();

// What the server gives every document, and the document itself.
const app = (await (await fetch("/app.json")).json()) as { request: Omit<Request, "path" | "now">; modules: Record<string, string> };
const text = await (await fetch("/plan.comp.md")).text();
const opened = new OpenDocument({ ...app.request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: new Date().toISOString() });

// A built module loads in its frame from the server, checked against its file's hash.
const sandbox = createSandbox({
  base: "/compmd/",
  modules: ({ pack, version, path }) => {
    const place = `${pack}@${version}/${path}`;
    const integrity = app.modules[place];
    return integrity === undefined ? undefined : { url: new URL(`/compmd/packs/${place}`, location.href).href, integrity };
  },
});
const view = sandbox.open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);
```

`base` is where the frame documents are served, ending in `/`; `modules` gives `{ url, integrity }` for a module served by URL, `{ code }` (a `Blob`) for one the page holds already, checked (as `resolvePacks` gives them, [chapter 9](09-packs-from-npm.md)), or `undefined` for none, whose uses show their fallbacks. The frame loads a module by URL with `<script integrity>`, so a file that doesn't match its hash never runs.

The HTML:

```html snippet=other-hosts/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>CompMD on a server of its own</title>
  </head>
  <body>
    <main id="page" style="height: 24rem"></main>
    <script type="module" src="./page.ts"></script>
  </body>
</html>
```

And the document, which imports the Mermaid pack the server gives:

````md snippet=other-hosts/plan.comp.md
---
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
---

# Where a page's parts come from

The page, its documents and the sandbox's frames come from one server:

```mermaid {alt="The server gives the page, the frames and the built modules"}
graph LR
  Server --> Page
  Server --> Frames
  Server --> Modules
```
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/other-hosts) in the playground.

## Run it

`public/` is what the server serves: the bundled page, the frames under `public/compmd/`, and the documents.

1. Bundle `index.html` and `page.ts` into `public/` with the bundler your app uses. The processor's `init` loads its WebAssembly file, `compmd_wasm_bg.wasm`, from beside its own module, so the bundler must copy that file with the code; `@compmd/processor` exports it as `@compmd/processor/compmd_wasm_bg.wasm` for a bundler or server that needs its path.
2. Copy `plan.comp.md` into `public/`.
3. `node build.ts public/compmd`, after the bundler, which may empty `public/`.
4. `node server.ts public 8080 compmd.config.json`, and open the address it prints.

The page shows the heading, the sentence and the diagram, drawn in a strict frame by a module the server served. The guides' tests run these steps with Vite as the bundler alone, with no CompMD plugin, and check each claim of this chapter.

## The registry's proxy in any server

With `registries` and `"fetch": "host"`, a page fetches packs it doesn't have only through its own server, so the registry learns nothing about readers ([chapter 9](09-packs-from-npm.md)). `registryProxy({ registries, base })` from `@compmd/registry` is the proxy, as a fetch handler, `(request: Request) => Promise<Response>`:

- `GET <base><name>` gives the pack's short metadata from its scope's registry, each tarball's URL rewritten to the proxy's own;
- `GET <base>-/tarball/<name>/<version>` streams that version's tarball;
- every other request is 404, and a registry that fails is 502.

It passes on nothing the reader sent, cookies least of all, and caches nothing itself, so put an HTTP cache or a CDN in front of it. A private registry's token goes in its `NpmRegistry`'s `headers`, on the server. `toNodeListener` from `@compmd/registry/node` turns any such handler into a Node request listener, for `http.createServer` as here, or for Express and Connect middleware; a worker or a server with fetch handlers (Deno, Bun, Cloudflare Workers) uses the handler as it is:

```ts
import { NpmRegistry, registryProxy } from "@compmd/registry";

export default { fetch: registryProxy({ registries: { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) }, base: "/compmd/registry/" }) };
```

The page then resolves its documents' packs with registries that point at the proxy, `{ "*": new NpmRegistry({ url: "/compmd/registry/" }) }`, as `virtual:compmd`'s `registry` does in a Vite app.

## Trusted components

The plugin builds an app's own trusted components with the app and gives them to the sandbox as `trusted`. Without it, import each component in your page, compiled by your bundler, and wrap it in its binding: `react` from `@compmd/render/react` for React, `svelte` from `@compmd/render/svelte` for Svelte (`@compmd/vite/react` and `@compmd/vite/svelte` give the same), a plain mount as it is. Then open the sandbox in host mode, `createSandbox({ base, page: "host", trusted: { stars: react(Stars) }, modules })`, and give the processor the folder pack as a release with `{ level: "trusted" }` in its policy. [The sandbox's README](../../sandbox/README.md) has every option, and [chapter 10](10-your-apps-own-components.md) the components themselves.

Next: [the Rust crate](19-the-rust-crate.md), processing documents outside a browser.
