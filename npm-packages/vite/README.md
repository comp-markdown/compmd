<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/branding/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/vite

A Vite plugin that gives a web app's [CompMD](https://compmd.dev/) documents their components with one line of configuration. It reads `compmd.config.json`, gives the processor every pack the app installs, serves the sandbox's frame documents and the packs' built modules from the app's own origin, and builds the app's own trusted components with the app. The design is [docs/vite.md](https://github.com/comp-markdown/compmd/blob/main/docs/vite.md).

**Website:** [compmd.dev](https://compmd.dev/), with the spec and a playground to try CompMD in.

```sh
npm install @compmd/vite @compmd/processor @compmd/sandbox
```

It implements the draft of the [CompMD specification](https://compmd.dev/spec/), version 1, and shares its version with `@compmd/render`, `@compmd/processor` and `@compmd/sandbox` ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)). It needs Vite 8 or later.

**Versions:**
- **0.12.0:** a page with a configuration of its own, in its folder ([the design](https://github.com/comp-markdown/compmd/blob/main/docs/vite-per-page-config.md)): the nearest configuration above a module that imports `virtual:compmd`. A project with one configuration is as it was. The configuration takes `editor.suggest`, the scopes an editor suggests packs from. W104 comes from the processor's component reader, which the app no longer needs TypeScript or Svelte's compiler for: `typescript` isn't a peer dependency any more. The configuration, the packs found and the lock file are read by `compmd-project`, in `@compmd/host`'s module, so the plugin and the Rust hosts read a project alike; an installed package that isn't scoped is no longer listed with its integrity, since no pack is unscoped. The build warns of a name two ambient packs export, naming both packs, since a document that uses it gets F023 and the component's fallback (spec §6.3). With registries, the bundle's `manifest.json` holds `registries`, `fetch` and `maxPackSize`, for a host whose own `registry/` proxy passes on to them. A pack's `compmd` field may be an object, which names its document by `document` (`documentOf`). On the sandbox's 0.12.0. Breaking, for a page or a worker that uses `@compmd/vite/config`: it loads `@compmd/host`'s module first (`await init()` from `@compmd/host`); in Node nothing changes.
- **0.11.0:** a pack's files are confined to the pack, read by one module (`@compmd/vite/pack-folder`): a path with `\`, `:` or NUL, or one that leaves the pack, is refused; `files` is read as npm reads it, globs and folders, and deny ranges as the processor reads them. Dev rebuilds are queued one after another (`serialized`), their errors logged, and adding or removing a file reloads. On the sandbox's 0.11.0, whose frames speak its protocol's version 4. Breaking, for an app's packs: a folder pack without `files` publishes only what its documents name; a definition path that interpolates after literal text needs `files`, or the build stops; an installed package whose `name` isn't its dependency's name is a `PackError`; an app's page and the frames are upgraded together.
- **0.10.0:** `loadApp(root, given, { analysis: false })` leaves out the analysis of the listed packs' components (W104), which loads TypeScript, for a caller that only processes documents with the app's `request`; on the sandbox's 0.10.0, whose frames speak its protocol's version 3 and lay a use out as in the page, with a document's own fonts and a pack's images, and which shows a registry pack's images from `packImages` (`resolvePacks`'s `images`; `packFiles` and `files` from 0.11.0, as the example below gives it).
- **0.9.0:** `origins` in `compmd.config.json`, the origins a host allows its documents' images and data from, or `"any"` for a host's own documents, given to the processor and to `virtual:compmd`'s sandbox; on the sandbox's 0.9.0, whose frames ask the page to highlight their code, so a page in frame mode loads `highlight.js` too, which `cspFor` allows.
- **0.8.0:** `cspFor` adds `'wasm-unsafe-eval'` to `script-src`, so a host-mode page highlights code with Oniguruma; the dev server serves the bundle with `@compmd/host`'s `respond`, as every host does; the build reports a listed pack's components against their signatures (W104). An app whose own policy doesn't come from `cspFor` adds `'wasm-unsafe-eval'` itself, or keeps Shiki's JavaScript engine.
- **0.7.1:** the same as 0.7.0; its homepage and links name the website's new address, [compmd.dev](https://compmd.dev/).
- **0.7.0:** the bundle ([hosting](https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md)): the frames under a hash of their files, `frames/<hash>/`, each pack's files by its place with its source components compiled ahead, and `manifest.json`; `out` and `source` for a desktop app's adapter; `buildBundle` for a host without Vite; `cspFor`; a `_headers` file; every file served with the serving contract's headers; `virtual:compmd`'s paths relative to the bundle's origin. Breaking: the frames move from `compmd/strict.html` to `compmd/frames/<hash>/`, and `virtual:compmd`'s `sandbox` gives `frames` and an `origin`, no longer a `base`. An app that uses `virtual:compmd` and `cspFor` changes nothing; one that wrote `compmd/` paths into its own policy or server rules updates them.

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # ESM and types into dist/
pnpm test            # unit tests in Node, and the example app in Chrome and WebKit
pnpm lint            # eslint and the type check
```

The example's tests drive the installed Chrome and Playwright's WebKit (`pnpm exec playwright install webkit`).

## Use it

```ts
// vite.config.ts
import react from "@vitejs/plugin-react";
import { compmd } from "@compmd/vite";
import { defineConfig } from "vite";

export default defineConfig({ plugins: [react(), compmd()] });
```

```ts
// where the app shows a document
import { OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";

const document = new OpenDocument({ ...request, path: "notes.comp.md", files, now: new Date().toISOString() });
const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: document.components });
view.render(document.expanded);
```

`request` holds the app's packs, the ambient ones and the policy, for the processor; `sandbox` holds where the frame documents are, the trusted components and where each built module is, for the sandbox. For their types, add `"@compmd/vite/client"` to `compilerOptions.types`.

## The configuration

`compmd.config.json` next to the app's `package.json`, or the `compmd` key of `package.json` as an object (not both; a string there names a pack's document, spec §6.4, and is no configuration). A tool that serves a page of its own, as `@compmd/pack`'s test page does, gives it in the plugin's options instead, `compmd({ config: { packs: { "./": {} } } })`, checked the same way, with folder packs relative to Vite's root. Every installed pack is available without it, in the strict sandbox: the configuration lists only the packs that get more, or that every document gets without importing them.

```json
{
  "$schema": "./node_modules/@compmd/vite/compmd.config.schema.json",
  "packs": {
    "@kit/mermaid": { "version": "^1.0.0" },
    "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com"] },
    "@acme/reviews": { "trust": "permissive", "host": true },
    "./components": { "trust": "trusted", "ambient": true }
  },
  "imports": "any",
  "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }],
  "maxPackSize": "5 MB",
  "libraries": ["d3"],
  "registries": { "*": "https://registry.npmjs.org/" },
  "fetch": "host"
}
```

| Key | What it is |
|---|---|
| `packs` | by pack name, or by folder path for a pack in the app |
| `packs.*.trust` | `strict` (the default), `permissive` (the network to its origins, and `host` if given), or `trusted`, for a folder in the app only |
| `packs.*.network` | the origins a permissive pack may reach, each `https://host[:port]` |
| `packs.*.host` | whether a permissive pack's components get `host` (spec §7.4) |
| `packs.*.ambient` | every document gets the pack's names without importing it |
| `packs.*.version` | a semver range the installed version must satisfy |
| `imports` | `any` (the default): a document may import any installed pack; `listed`: only those listed |
| `deny` | packs denied, all versions or a range, with the reason readers see (F026) |
| `maxPackSize` | a pack whose files are larger is left out: bytes, or a size such as `"5 MB"`; for a pack from a registry, its unpacked size, before it is downloaded (10 MB there when not given) |
| `libraries` | npm packages, installed in the app, that the frames offer components through their import map, besides React and Svelte: `"d3"`, or `{ "name": "d3", "imports": ["d3", "d3-scale"] }` |
| `registries` | the npm registries packs the app doesn't install come from, by scope (`"@kit"`), with `"*"` for every other scope, each named in full |
| `fetch` | who reaches them: `host` (the default), through the app's proxy at `compmd/registry/`; `direct`, the reader's browser |
| `origins` | the origins documents' remote data and images come from without asking the reader, each `https://host[:port]`; or `"any"`, every origin, for documents the app publishes itself, such as a static site's or a blog's (spec §15). None by default: the reader is asked about each |

**A page with a configuration of its own.** A Vite project may hold pages whose documents different people write, such as a site's own pages and a playground where anyone writes. A folder with its own `compmd.config.json` (or `package.json`'s `compmd` object) gives the pages there their own pack settings: a module that imports `virtual:compmd` gets the nearest configuration above it, up to Vite's root ([the design](https://github.com/comp-markdown/compmd/blob/main/docs/vite-per-page-config.md)).
- **Nothing passes between configurations.** A pack a nested configuration doesn't list is strict there, whatever the root gives it, and its `deny`, `origins` and registries are its own. Its folder paths are relative to its own folder.
- **One configuration to a page.** A page whose modules import two configurations' `virtual:compmd` stops the build, naming the modules; the dev server shows the same in the page.
- **One bundle for all.** Each pack's files are served once, compiled for the frames when a configuration doesn't trust the pack; one build of the frames for each distinct `libraries`; one registry proxy, so two configurations can't send one scope to different registries. With `out`, the manifest holds the root's configuration.

`@compmd/vite/config` reads and translates the same configuration without Vite or Node, for a host that isn't a Vite app: `readConfig(json)` checks it and lists every problem, and `policyOf(config, nameOf)` gives the processor's `policy`, `ambient` and `allowedOrigins`.

`@compmd/vite/pack-folder` reads a pack's folder, in Node, as the plugin and `@compmd/pack` both do: its manifest (`identityOf`, `documentOf`), what its documents name in the pack (`referencesOf`, `reachOf`), each path confined to the pack (`joinedInPack`: no `..` out of it, no `\`, `:` or NUL), and what it publishes (`publishedFiles`, `files` read as npm reads it).

## The app's own components

A folder of the app is a pack: a `package.json` with a name, and a document that imports the components and states their signatures.

````md
<!-- components/index.comp.md -->
---
md:
  import:
    - from: ./Stars.tsx
      names: [{ name: Stars, as: stars }]
  export:
    pack: "@acme/app"
    names: [stars]
---

```compmd component=stars signature
params:
  value: input<number>
```
````

Listed as `trusted`, its components are compiled by the app's own build, with its React or Svelte plugin, and mount in the app's page: `.jsx` and `.tsx` with React, `.svelte` with Svelte, `.js` and `.ts` as plain mounts. They treat their params as untrusted input, since documents give them.

## What it serves

The plugin writes the bundle ([hosting](https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md), decision 3), which a web app serves under its base, in `compmd/` (the `folder` option changes it):

- `frames/<hash>/`, the sandbox's frame documents and their files (`buildFrames`), with the libraries the configuration offers, under a hash of their files: a new build is a new path, so no cache serves an old frame;
- `packs/<name>@<version>/<path>`, each installed and folder pack's files, as published (a denied pack's none): what its `package.json`'s `files` matches inside the pack, what npm always adds, and every file its documents name. A folder pack without `files` publishes only what its documents name (their imports, data, fonts' files, and the images and links in their definitions), so the build stops when a definition names a path that interpolates after literal text (`./icons/{{icon}}.svg`), which `files` must list; an installed one without `files` is its whole folder, as npm published it; the frames load its built modules checked against the hash `sandbox.modules` gives;
- `registry/`, with registries and `fetch: "host"`, the proxy (below), on the dev server and `vite preview` only.

The dev server serves them; the build writes them into the output, with a `_headers` file for a static host that reads one. Every file is served with the serving contract's headers (`Content-Security-Policy: sandbox allow-scripts`, `Access-Control-Allow-Origin: *`, `Access-Control-Expose-Headers: *`, `X-Content-Type-Options: nosniff`, `Cache-Control: max-age=31536000, immutable`), as the dev server and `vite preview` do: the frames have an opaque origin, and a frame document is sandboxed even when opened on its own. `cspFor(origin, config)` gives what the app's page adds to its own content security policy.

**Since 0.7.0** the frames are under `frames/<hash>/`, no longer at `compmd/strict.html`: an app that uses `virtual:compmd` and `cspFor` changes nothing; one that wrote `compmd/` paths into its own policy or server rules updates them.

### For a desktop app, or a host without Vite

`compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })` writes the bundle, with its `manifest.json`, into `out` for an adapter to embed or serve; the dev server then serves none, and the page takes the origin from the adapter (`globalThis.__COMPMD__`). `source` is the custom scheme the bundle is served at, which the frames' policies name. A host built without Vite calls `buildBundle({ out, source })` from its build instead.

## What it checks

At build start, and when the configuration, `package.json` or a pack's files change in dev:

- the configuration, with every problem at once;
- that each listed pack is installed, at the version the configuration asks, and each library;
- each listed pack's documents, through the processor: an error stops the build, a warning is Vite's;
- each listed pack's React and Svelte components against their signature fences, with the processor's component reader (`@compmd/processor/components`): a declared param or slot with no prop, a prop whose TypeScript type doesn't take the declared one, or a prop the signature doesn't declare is a W104 warning. A component whose props it can't read is left unchecked.

Installed packs come from `node_modules`. Each one's `integrity` comes from the app's lock file (`pnpm-lock.yaml`, every one of its documents, or `package-lock.json`), for documents that pin one; a pack installed from a registry that the lock file has no integrity for is a warning, since documents that pin its `integrity` won't load it (F035). Without `registries`, a document that imports a pack the app doesn't install gets F026.

## Packs from a registry

With `registries`, a document may import a pack the app doesn't install. The app installs `@compmd/registry` too, and resolves the document's packs before processing it ([docs/vite.md](https://github.com/comp-markdown/compmd/blob/main/docs/vite.md#packs-from-a-registry)):

```ts
import { defaultCache, resolvePacks } from "@compmd/registry";
import { registry, request, sandbox } from "virtual:compmd";

const asked = { ...request, path: "notes.comp.md", files, now: new Date().toISOString() };
const resolved = registry && (await resolvePacks(asked, { ...registry, cache: defaultCache() }));
const document = new OpenDocument(resolved?.request ?? asked);
const view = createSandbox({ ...sandbox, page: "host", modules: (place) => resolved?.modules(place) ?? sandbox.modules(place), packFiles: (place) => resolved?.files(place) }).open(container, { components: document.components });
```

`registry` holds what `resolvePacks` takes from the configuration: the registries, who fetches, and the size cap.

- **`fetch: "host"`** (the default): the page reaches the registries only through the app's proxy, `compmd/registry/`, so they learn nothing about readers. The dev server and `vite preview` serve it. A deployed app serves it from its server or a worker, with `@compmd/registry`'s `registryProxy({ registries, base: "/compmd/registry/" })` and the same registries.
- **`fetch: "direct"`**: the reader's browser fetches from the registries named, which then learn the reader's address and the packs' names. Tell your readers. It is for hosts with no server.

## The example

[`example/`](https://github.com/comp-markdown/compmd/tree/main/npm-packages/vite/example) is an app with its own `stars` as a trusted folder pack, a strict pack with a built module installed as a dependency, and a pack its configuration denies. Its second page, `readers/`, has a configuration of its own, which lists the app's components without trust, so they run in frames there. `pnpm --dir example dev` runs it; the tests build it and drive it in Chrome and WebKit. It renders in host mode, and with `?page=frame` in one strict frame.

On the dev server it opens the [Developer Tools' inspector](https://github.com/comp-markdown/compmd/blob/main/npm-packages/devtools/README.md#the-inspector) under its page, on its view: a build leaves it out, since it is imported only in the dev server's mode. Its config lists `@compmd/devtools` in `optimizeDeps.exclude`, so Vite builds the inspector's workers. The tests check the inspector on the dev server, in Chrome and WebKit, in both page modes.

## License

MIT.
