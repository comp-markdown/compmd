# 18. Other hosts

`@compmd/vite` does five jobs for a Vite app. It reads the configuration. It builds the **bundle**: the sandbox's frames, the packs' files and a manifest. It serves the bundle with the headers the frames need. It gives the page the processor's request and the sandbox's paths, as `virtual:compmd`. And it serves the registry proxy. An app built another way does them itself, with the same functions the plugin uses. This chapter builds such an app: a Node server with no Vite plugin, which serves a page, its document, the bundle and the registry proxy, and a page that draws a diagram with the Mermaid pack. A desktop app has a plugin of its own for this ([chapter 19](19-a-tauri-app.md)).

## What you do by hand

| The plugin's job | Without it |
|---|---|
| reads and checks `compmd.config.json` | `buildBundle` reads it; `readConfig` from `@compmd/vite/config` for the rest |
| builds the bundle: the frames, each pack's files with its source components compiled, `manifest.json` | `buildBundle`, from `@compmd/vite` |
| serves the bundle with the serving contract | your server, with `CONTRACT_HEADERS` from `@compmd/sandbox/check` |
| gives the processor the packs and the policy (`request`), and the sandbox the bundle's frames and modules (`sandbox`) | what your server makes from `manifest.json` |
| serves the registry proxy | `registryProxy`, with `toNodeListener` for Node |
| builds trusted components with the app (`sandbox.trusted`) | your bundler, and the bindings of `@compmd/render` |

And one thing every host does, with the plugin or without it: it runs the hosting check, `runCheck`, in the browser its readers use.

The example has eight files: the configuration, a build script for the bundle, the server, the page and its HTML, the check and its HTML, and a document.

```sh
npm install @compmd/processor @compmd/sandbox @compmd/registry @compmd/vite @comp-markdown/mermaid
```

`@compmd/vite` is here for `buildBundle` and its configuration functions, which need no Vite server or build to run; npm installs Vite with it all the same, as a peer dependency.

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

- **`readConfig(value, where?)`** checks the parsed JSON and gives it with every default filled in, sizes in bytes and origins normalized (`ReadConfig`). What is wrong throws a `ConfigError`, whose `problems` lists every problem at once and whose message starts with `where` (`compmd.config.json` by default). [The configuration's reference](21-the-configurations-reference.md) has every key and every problem.
- **`policyOf(config, nameOf, refused?)`** gives what the processor takes: `policy` (the packs with their trust, `imports`, `deny`, and `refused` when you give it) and `ambient`, the names of the ambient packs. `nameOf` turns a key into a pack name: a pack name is its own, and a folder pack (`./components`) is its `package.json`'s `name`. `refused` holds the packs you leave out, each with why; the plugin puts there the packs larger than `maxPackSize`.

The module also exports `isFolder(key)`, which tells a folder key from a pack name, and the types `Config`, `ReadConfig`, `PackOptions`, `Library` and `TrustLevel`. A host on the bundle needs `policyOf` only for documents it processes before it has a bundle: the manifest carries the policy.

## The bundle

The bundle is everything built before anyone reads, in one folder:

```text
bundle/
  frames/<hash>/      strict.html, permissive.html, the frame runtime, React, React DOM, Svelte, Shiki, Temml
  packs/<name>@<version>/   each pack's files, its source components compiled beside them (<path>.js)
  manifest.json       the frames' hash, each pack with its modules' hashes, the policy, the ambient packs
  _headers            the serving contract's headers, for a static host that reads this file
```

The frames are under a hash of their files, so a new build is a new path and no cache ever serves an old frame. They are built from the `@compmd/sandbox` your page imports, so the page and its frames always speak the same protocol. `buildBundle` writes it, at build time, in Node, from the app's folder (the one with its `package.json` and `node_modules`, where it finds the installed packs):

```ts snippet=other-hosts/build.ts
// Builds the bundle the server serves, from the app's folder, the one with its `package.json` and `node_modules`:
// `node build.ts bundle compmd.config.json`.
import { readFile } from "node:fs/promises";
import { buildBundle } from "@compmd/vite";

const [out = "bundle", configFile = "compmd.config.json"] = process.argv.slice(2);
const config: unknown = JSON.parse(await readFile(configFile, "utf8"));
const { manifest, warnings } = await buildBundle({ out, config });
for (const warning of warnings) console.warn(warning);
console.log(`The bundle is in ${out}: the frames at frames/${manifest.frames.hash}/, and ${manifest.packs.map(({ pack, version }) => `${pack}@${version}`).join(", ")}.`);
```

`buildBundle({ out, root?, config?, source? })` takes the folder to write (only `frames/`, `packs/`, `manifest.json` and `_headers` there are its), the app's folder (the working directory by default), the configuration (`compmd.config.json` in `root` by default), and `source`, for a bundle served on a custom scheme ([chapter 19](19-a-tauri-app.md)). It checks the app as the plugin does, and throws, saying what to fix, where the plugin stops a build. It gives the `manifest` it wrote, and `warnings` that don't stop it, such as a pack the lock file has no integrity for.

## The server

```ts snippet=other-hosts/server.ts
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
```

What it does, part by part:

- **The bundle's origin.** The server serves the bundle under `/compmd/` on the page's own origin, as a Vite app does. That is enough on the web: a frame's `sandbox` attribute gives it an opaque origin wherever its files come from. A site that shows documents others wrote serves the bundle from a domain of its own instead, as code hosts serve user content, and the page names that origin.
- **The serving contract.** Every file of the bundle is served with the same five headers, `CONTRACT_HEADERS`: `Content-Security-Policy: sandbox allow-scripts`, so a frame document opened on its own, outside a frame, is sandboxed all the same; `Access-Control-Allow-Origin: *` and `Access-Control-Expose-Headers: *`, since a frame loads its files cross-origin; `X-Content-Type-Options: nosniff`; and `Cache-Control: max-age=31536000, immutable`, since a path's file never changes. Only the frames of the bundle's hash and the files of its packs are served. Anything else is a `404`, the manifest too, which is the host's and not the frames', with `Access-Control-Allow-Origin: *`, so a page can read why.
- **The packs.** The processor never reads a disk or a registry: a pack is in the request's `packs`, by name, as a list of `PackRelease`s. The server makes each from the bundle's copy of the pack: its `package.json`, its document (its `compmd` field names it, `index.comp.md` by default), and each built module as empty text, since the processor needs only to know it is there. A pack whose document imports other files, or a source component whose text the processor reads, needs those files too. A release's `integrity` is the hash the registry published for the version's tarball, which `buildBundle` read from the app's lock file; a document that pins its import with `integrity` gets the pack only if the two are equal (F035 otherwise).
- **The policy.** The manifest's `policy` is `policyOf`'s, with the frames' runtimes and libraries by name and version, so the processor reports a component on a runtime the frames don't have (W102) and a pack whose `peerDependencies` they can't meet (W121).
- **`/app.json`** is what `virtual:compmd` holds in a Vite app: the request's `packs`, `ambient` and `policy`, the bundle's `frames`, and each built module's path under the origin with its file's hash. A server that renders many documents makes it once, as here.

## The page

The page opens a document with what the server gives, and renders it in frame mode:

```ts snippet=other-hosts/page.ts
import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

await init();

/** What the server gives every page: the request's packs and policy, and where the bundle's frames and modules are. */
export interface App {
  request: Omit<Request, "path" | "now">;
  frames: string;
  modules: Record<string, { path: string; integrity: string }>;
}

const app = (await (await fetch("/app.json")).json()) as App;
const text = await (await fetch("/plan.comp.md")).text();
const opened = new OpenDocument({ ...app.request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: new Date().toISOString() });

// The bundle is served under `compmd/` on this page's origin. A built module loads in its frame by its path there,
// checked against its file's hash.
const sandbox = createSandbox({
  origin: "/compmd/",
  frames: app.frames,
  modules: ({ pack, version, path }) => app.modules[`${pack}@${version}/${path}`],
});
const view = sandbox.open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);
```

`origin` is where the bundle is served, a full URL or one relative to the page; without it, the sandbox takes the one a host's adapter gave the page, `globalThis.__COMPMD__.origin`, as a desktop app's plugin does. `frames` is the bundle's frames under it, `frames/<hash>/`. `modules` gives, for each built module the processor names by its place, `{ path, integrity }`, its path under the origin and its file's hash; or `{ url, integrity }` for one served elsewhere; or `{ code }` (a `Blob`) for one the page holds already, checked (as `resolvePacks` gives them, [chapter 9](09-packs-from-npm.md)); or `undefined` for none, whose uses show their fallbacks. The frame loads a module by URL with `<script integrity>`, so a file that doesn't match its hash never runs.

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

And the document, which imports the Mermaid pack the bundle has:

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

## The hosting check

A host knows it serves the bundle as the sandbox needs only by trying it, in the browser its readers use. `runCheck` from `@compmd/sandbox/check` draws a small document of its own in the page, and asks the host for the frames' files:

```ts snippet=other-hosts/check.ts
// The hosting check, in the browser the app's readers use: the server serves the bundle as the sandbox needs.
import { runCheck } from "@compmd/sandbox/check";
import type { App } from "./page.ts";

const app = (await (await fetch("/app.json")).json()) as App;

// The Mermaid pack's module, which must load by its URL and draw.
const [place = ""] = Object.keys(app.modules);
const at = place.lastIndexOf("@");
const [version = "", ...path] = place.slice(at + 1).split("/");
const mermaid = { pack: place.slice(0, at), version, path: path.join("/") };

const report = await runCheck({
  container: document.getElementById("check") as HTMLElement,
  origin: "/compmd/",
  frames: app.frames,
  modules: ({ pack, version, path }) => app.modules[`${pack}@${version}/${path}`],
  packs: [{ place: mermaid, params: { code: "graph LR; A --> B" } }],
});
(document.getElementById("report") as HTMLElement).textContent = report.message;
document.body.dataset.checked = report.ok ? "passed" : "failed";
```

```html snippet=other-hosts/check.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>The hosting check</title>
  </head>
  <body>
    <section id="check" style="height: 16rem"></section>
    <p id="report" role="status"></p>
    <script type="module" src="./check.ts"></script>
  </body>
</html>
```

It checks that a strict frame loads from the origin, and that in it the origin is opaque, `fetch` is refused, and there is no host API (`hostApi` names the globals to look for, such as a desktop framework's) and no WebKit `ipc` message handler; that a code block is highlighted, so the page loads the frames' `highlight.js`; that each pack in `packs` loads by its URL and draws; that the frames' files arrive with exactly the contract's headers; and that a path outside the contract is a `404` the page can read. Its report has `ok`, every check that `passed`, every one that `failed` with why, and a one-line `message`, for a test, an exit code or a status page. A page with a content security policy allows `connect-src` and `script-src` to the frames for it (`cspFor`, [chapter 11](11-the-sandbox-closer.md)), as it must anyway.

## Run it

`public/` is what the server serves as the app: the bundled pages and the document. The bundle is beside it, in `bundle/`.

1. Bundle `index.html`, `check.html` and their scripts into `public/` with the bundler your app uses. The processor's `init` loads its WebAssembly file, `compmd_wasm_bg.wasm`, from beside its own module, so the bundler must copy that file with the code; `@compmd/processor` exports it as `@compmd/processor/compmd_wasm_bg.wasm` for a bundler or server that needs its path.
2. Copy `plan.comp.md` into `public/`.
3. `node build.ts bundle compmd.config.json`, from the app's folder.
4. `node server.ts public bundle 8080 compmd.config.json`, and open the address it prints, and `check.html` beside it.

The page shows the heading, the sentence and the diagram, drawn in a strict frame by a module the server served, and the check page says `passed:` with each check. The guides' tests run these steps with Vite as the bundler alone, with no CompMD plugin, and check each claim of this chapter, in Chrome and WebKit.

## Frames a host builds itself

A host that can't use the bundle (one that serves frames another tool built, or a page of an older layout) builds the frame documents alone with `buildFrames` from `@compmd/sandbox/build`, and gives `createSandbox` their `base`, where they are served, ending in `/`, in place of `origin` and `frames`:

```ts
import { buildFrames, buildLibraries, readLibraries, RUNTIMES } from "@compmd/sandbox/build";

const imports = await buildLibraries([{ name: "d3", imports: ["d3"] }], { root, out: "public/sandbox/libs", shared: RUNTIMES });
const libraries = await readLibraries("public/sandbox/libs");
const definition = await buildFrames({ out: "public/sandbox", imports, libraries });
// in the page: createSandbox({ base: "/sandbox/", modules })
```

`buildFrames` gives the frames' **library definition**, the runtimes and libraries they have, each with its version and import names, whose names and versions are the policy's `runtimes` and `libraries`, and writes it beside them as `library-definition.json`, which `libraryDefinition(out)` reads back. `buildLibraries` bundles the libraries you offer besides the runtimes, with Vite's bundler, so it needs Vite installed; build them before the frames, since `buildFrames` copies the runtimes' files over the folder. Such a host serves the frames with the contract's headers too, keeps their path apart for each build, and compiles no pack's sources ahead: the sandbox compiles them in each reader's page. The bundle does all of that for you.

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

The plugin builds an app's own trusted components with the app and gives them to the sandbox as `trusted`. Without it, import each component in your page, compiled by your bundler, and wrap it in its binding: `react` from `@compmd/render/react` for React, `svelte` from `@compmd/render/svelte` for Svelte (`@compmd/vite/react` and `@compmd/vite/svelte` give the same), a plain mount as it is. Then open the sandbox in host mode, `createSandbox({ origin, frames, page: "host", trusted: { stars: react(Stars) }, modules })`, and give the processor the folder pack as a release with `{ level: "trusted" }` in its policy. [The sandbox's README](../../sandbox/README.md) has every option, and [chapter 10](10-your-apps-own-components.md) the components themselves.

Next: [a Tauri app](19-a-tauri-app.md), CompMD in a desktop app.
