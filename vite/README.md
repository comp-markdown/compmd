<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/vite

A Vite plugin that gives a web app's CompMD documents their components with one line of configuration. It reads `compmd.config.json`, gives the processor every pack the app installs, serves the sandbox's frame documents and the packs' built modules from the app's own origin, and builds the app's own trusted components with the app. The design is [docs/vite.md](../docs/vite.md).

**Website:** [compmd.hakanserce.com](https://compmd.hakanserce.com/), with the spec and a playground to try CompMD in.

```sh
npm install @compmd/vite @compmd/processor @compmd/sandbox
```

It implements the draft of the [CompMD specification](https://compmd.hakanserce.com/spec/), version 1, and shares its version with `@compmd/render`, `@compmd/processor` and `@compmd/sandbox` ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)). It needs Vite 8 or later.

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

`@compmd/vite/config` reads and translates the same configuration without Vite or Node, for a host that isn't a Vite app: `readConfig(json)` checks it and lists every problem, and `policyOf(config, nameOf)` gives the processor's `policy` and `ambient`.

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

Under the app's base, in `compmd/` (the `folder` option changes it):

- `strict.html`, `permissive.html` and their files, the sandbox's frame documents (`buildFrames`), with the libraries the configuration offers;
- `packs/<name>@<version>/<path>`, each installed pack's built modules (the `.js` files its documents import), which the frames load checked against the hash `sandbox.modules` gives;
- `registry/`, with registries and `fetch: "host"`, the proxy (below), on the dev server and `vite preview` only.

The dev server serves them; the build writes them into the output. The frames have an opaque origin, so a host must serve this folder with `Access-Control-Allow-Origin: *`, and the frame documents with `Content-Security-Policy: sandbox allow-scripts`, as the dev server and `vite preview` do.

## What it checks

At build start, and when the configuration, `package.json` or a pack's files change in dev:

- the configuration, with every problem at once;
- that each listed pack is installed, at the version the configuration asks, and each library;
- each listed pack's documents, through the processor: an error stops the build, a warning is Vite's.

Installed packs come from `node_modules`. Each one's `integrity` comes from the app's lock file (`pnpm-lock.yaml`, every one of its documents, or `package-lock.json`), for documents that pin one; a pack installed from a registry that the lock file has no integrity for is a warning, since documents that pin its `integrity` won't load it (F035). Without `registries`, a document that imports a pack the app doesn't install gets F026.

## Packs from a registry

With `registries`, a document may import a pack the app doesn't install. The app installs `@compmd/registry` too, and resolves the document's packs before processing it ([docs/vite.md](../docs/vite.md#packs-from-a-registry)):

```ts
import { defaultCache, resolvePacks } from "@compmd/registry";
import { registry, request, sandbox } from "virtual:compmd";

const asked = { ...request, path: "notes.comp.md", files, now: new Date().toISOString() };
const resolved = registry && (await resolvePacks(asked, { ...registry, cache: defaultCache() }));
const document = new OpenDocument(resolved?.request ?? asked);
const view = createSandbox({ ...sandbox, page: "host", modules: (place) => resolved?.modules(place) ?? sandbox.modules(place) }).open(container, { components: document.components });
```

`registry` holds what `resolvePacks` takes from the configuration: the registries, who fetches, and the size cap.

- **`fetch: "host"`** (the default): the page reaches the registries only through the app's proxy, `compmd/registry/`, so they learn nothing about readers. The dev server and `vite preview` serve it. A deployed app serves it from its server or a worker, with `@compmd/registry`'s `registryProxy({ registries, base: "/compmd/registry/" })` and the same registries.
- **`fetch: "direct"`**: the reader's browser fetches from the registries named, which then learn the reader's address and the packs' names. Tell your readers. It is for hosts with no server.

## The example

[`example/`](example) is an app with its own `stars` as a trusted folder pack, a strict pack with a built module installed as a dependency, and a pack its configuration denies. `pnpm --dir example dev` runs it; the tests build it and drive it in Chrome and WebKit. It renders in host mode, and with `?page=frame` in one strict frame.

On the dev server it opens the [Developer Tools' inspector](../devtools/README.md#the-inspector) under its page, on its view: a build leaves it out, since it is imported only in the dev server's mode. Its config lists `@compmd/devtools` in `optimizeDeps.exclude`, so Vite builds the inspector's workers. The tests check the inspector on the dev server, in Chrome and WebKit, in both page modes.

## License

MIT.
