# The configuration and `@compmd/vite`

**Status: accepted (2026-10-30).** Builds H3 of [components from apps and packs](components-from-apps-and-packs.md#h3-configuration-filled-at-build-time-recommended-for-web-apps): a web app adds packs with `npm install` and a line of configuration, and a Vite plugin does the rest. The sandbox it feeds is [the sandbox](running-components-in-the-sandbox.md).

## Proposed decisions

1. **One configuration file, `compmd.config.json`** (or the `compmd` key of the app's `package.json`), the same for every kind of host: it lists the packs that get more than a strict sandbox or are ambient, which packs a document may import, the packs denied, and a size limit. A JSON schema ships with it. The plugin reads it at build; the playground reads the same format at run time.
2. **The configuration is written for people, then translated.** Trust is a word (`"trust": "permissive"`), not the processor's `{"level": …}`; a pack is a name or a folder path. One function, `policyOf(config)`, turns it into the processor's `Policy` and `ambient`, so every host translates it the same way.
3. **Every installed pack is available, strict by default.** The plugin gives the processor every dependency of the app that is a pack (its `package.json` has a `compmd` field, or the package has `index.comp.md`), so `npm install` is enough for a document to import it (P2). Listing a pack is only to give it more, or to make it ambient.
4. **Trusted only for a folder in the app.** `"trust": "trusted"` is allowed only on a path (`./components`), never on a registry name: the plugin stops the build otherwise. A trusted pack's components are imported by the app's own build, compiled by the app's Vite plugins (React, Svelte), and mounted inline.
5. **One virtual module gives the app everything.** `import { request, sandbox } from "virtual:compmd"`: `request` holds `packs`, `ambient` and `policy` for the processor; `sandbox` holds `base`, `trusted` and `modules` for `createSandbox`. The app spreads them in; it writes no policy, path or import map.
6. **The plugin writes the bundle** ([hosting](hosting.md), decision 3): the frame documents and their libraries under their content hash, `frames/<hash>/`, each pack's files at `packs/<name>@<version>/<path>`, and `manifest.json`. A web app serves it under one folder of the app, `compmd/`: the dev server with the serving contract's headers (hosting, decision 8), the build in the output, with a `_headers` file for a static host that reads one, and without the manifest, which is the host's. With `out`, it goes into that folder instead, for an adapter to embed or serve, and the dev server serves none (hosting, decision 2).
7. **What's wrong in the configuration or a pack stops the build or warns, at build time.** The plugin processes each listed pack's document once: errors (F) stop the build, warnings (W104, W117) are Vite warnings.
8. **Packs the app doesn't install come from the registries the configuration names** ([packs from a registry](packs-from-a-registry.md), R5), when it names any. `registries` names the registries themselves; `fetch` says who reaches them: the app's own proxy, under `compmd/registry/` (`host`, the default), or the reader's browser (`direct`). `virtual:compmd` gives `registry`, the options `resolvePacks` takes, and the dev and preview servers are the proxy.

## Terms

- **Configuration:** `compmd.config.json`, or the `compmd` key of `package.json` when it is an object. The plugin reads one; both is an error. A string there is a pack's document (spec §6.4), as in a pack's own folder, and no configuration.
- **Installed pack:** a dependency in the app's `node_modules` that is a CompMD pack (spec §6.4).
- **App pack:** a folder in the app's repository that is a pack: a `package.json` with a name, and a document that exports the app's own components.
- **Policy:** what the processor takes (`Request.policy`): packs with their trust, `imports`, `deny`, `refused`.

## The configuration

```json
{
  "$schema": "./node_modules/@compmd/vite/compmd.config.schema.json",
  "packs": {
    "@kit/mermaid": {},
    "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com"] },
    "@acme/reviews": { "trust": "permissive", "host": true },
    "./components": { "trust": "trusted", "ambient": true }
  },
  "imports": "any",
  "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }],
  "maxPackSize": "5 MB",
  "libraries": ["d3"],
  "registries": { "@kit": "https://npm.kit.example/", "*": "https://registry.npmjs.org/" },
  "fetch": "host"
}
```

| Key | What it is | Becomes |
|---|---|---|
| `packs` | a pack name or a folder path, each with its options | `policy.packs`, `ambient` |
| `packs.*.trust` | `strict` (the default), `permissive`, `trusted` (paths only) | the pack's level |
| `packs.*.network` | origins a permissive pack may reach, each `https://host[:port]` | `network` |
| `packs.*.host` | whether a permissive pack's components get `host` | `host` |
| `packs.*.ambient` | every document gets its names without importing it | `ambient` |
| `packs.*.version` | a semver range the installed version must satisfy, or the build stops | — |
| `imports` | `any` (the default) or `listed` | `policy.imports` |
| `deny` | packs denied, all versions or a range, with the reason readers see | `policy.deny` |
| `maxPackSize` | a pack whose files are larger is left out, with this reason; a pack from a registry, by its unpacked size, before it is downloaded (10 MB there when not given) | `policy.refused`; `registry.maxPackSize` |
| `libraries` | npm package names, installed in the app, that the frames offer components through their import map, besides React and Svelte | `buildLibraries`; with their versions, `policy.libraries` |
| `registries` | the npm registries packs the app doesn't install come from, by scope (`@kit`), with `*` for every other scope, each named in full (`https:`, or `http:` for localhost), with no credentials | `registry.registries` |
| `fetch` | who reaches them: `host` (the default), through the app's proxy; `direct`, the reader's browser | `registry.fetch` |

- A folder pack's name is its `package.json`'s `name`, which its document's `export.pack` matches (F022): the policy and `ambient` use that name, so documents can import `@acme/app` by name too (decision 7 of the design: a configured name is the host's).
- `ambient` is per pack, not a list of names, so a folder pack needs no second spelling.
- `network` takes origins only: a path, a wildcard or `http:` other than `localhost` stops the build.
- `libraries` names npm packages, as `npm install` does: `"d3"` is the package `d3`, which the app installs (`npm install d3`). The plugin builds each one once into `compmd/libs/`, and the frames' import map resolves a component's `import * as d3 from "d3"` to that copy, as the playground offers d3 today. A package whose components import more than its name gives its import names: `{ "name": "d3", "imports": ["d3", "d3-scale"] }`. A library that isn't installed stops the build.
- The request's `policy.runtimes` and `policy.libraries` are the frames' library definition (`@compmd/sandbox`'s `libraryDefinition`): the runtimes the frames have and the libraries the configuration offers, each with its version. So the processor reports a component on a runtime the frames don't have (W102) and a pack whose `peerDependencies` they can't meet (W121, spec §6.4), and a document's import of an offered library isn't portable (spec §6.3).

## What the plugin does

**At build start**, and again when the configuration, a pack folder or `node_modules` changes in dev:

1. Reads the configuration, checks it against the schema.
2. Finds the installed packs (decision 3) and the folder packs.
3. For each pack, reads `package.json`, its document and the files it imports, as the processor's `PackRelease`: version from `package.json`, integrity from the app's lock file (below), and its files, except built modules (`.js` files a pack document imports), which the processor names by place and the frames load by URL.
4. Packs larger than `maxPackSize` become `refused`, with the reason.
5. Processes each listed pack's document alone, and reports its diagnostics (decision 7).
6. Writes the bundle (decision 6) into a cache folder, or into `out`: the frames with `buildFrames` and the `libraries` with `buildLibraries` once; the packs' files, their source components compiled beside them (not a trusted pack's, which the app's bundler compiles; [hosting](hosting.md), decision 3), and the manifest each time. A component that doesn't compile stops the build.

**`virtual:compmd`**, for the app:

```ts
import { request, sandbox } from "virtual:compmd";
import { OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

const document = new OpenDocument({ ...request, path: "notes.comp.md", files, now: new Date().toISOString() });
const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: document.components });
view.render(document.expanded);
```

- `request.packs` holds each pack's documents and source files as text (small: signatures, CompMD, JSX), so the app needs no fetch to process a document.
- Its paths are relative to the bundle's origin: the adapter's, `globalThis.__COMPMD__.origin`, or else the app's `compmd/` folder (hosting, decision 9). `sandbox.base` is `frames/<hash>/` there.
- `sandbox.modules` maps a built module's place to `packs/<name>@<version>/<path>` on the origin and the file's `sha384`, which the plugin computes from the installed file; a source component's place maps to its compiled module, `<path>.js`, with a Svelte component's `css`.
- `sandbox.trusted` imports each trusted pack's component files and wraps them in their binding (`react(Stars)`, `svelte(Rating)`, a plain mount as it is), by the file's extension: a component file runs with its binding's default runtime, and an import has no `runtime` of its own (spec §6.1).
- `registry` holds what `resolvePacks` takes from the configuration (below), or is `undefined` when it names no registry.

## Hosts other than a web app

- **A desktop app built with Vite:** `compmd({ out: "src-tauri/compmd", source: "compmd://localhost/" })`. `out` is relative to Vite's root, and in it the plugin owns only `frames/`, `packs/`, `manifest.json` and `_headers`. `source` is the custom scheme's form, which the frames' policies name (`compmd://localhost/frames/`) and the manifest records, so the adapter serves the scheme it was built for.
- **A host built without Vite:** `buildBundle({ out, root, config, source })` writes the same bundle, with `_headers`, and gives the manifest and the checks' warnings; `root` is the working directory by default.
- **The page's content security policy:** `cspFor(origin, config)`, from `@compmd/vite` or `@compmd/vite/config`, gives the directives a host's page adds (hosting, decision 10), as `tauri.conf.json`'s `security.csp` takes them.

## Packs from a registry

With `registries`, a document may import a pack the app doesn't install ([packs from a registry](packs-from-a-registry.md)). The app resolves it before processing, as it fetches the document's data, and processes again when it arrives (decision 7 there):

```ts
import { defaultCache, resolvePacks } from "@compmd/registry";
import { registry, request, sandbox } from "virtual:compmd";

const asked = { ...request, path: "notes.comp.md", files, now: new Date().toISOString() };
const resolved = registry && (await resolvePacks(asked, { ...registry, cache: defaultCache() }));
const document = new OpenDocument(resolved?.request ?? asked);
const view = createSandbox({ ...sandbox, page: "host", modules: (place) => resolved?.modules(place) ?? sandbox.modules(place) }).open(container, { components: document.components });
```

- **`registries` names the registries themselves**, npm's or a private one, never a URL of the app: the proxy needs them to pass requests on to, and with `direct` the reader's browser fetches from them. A document or a pack never names one (companion, section 4).
- **`fetch: "host"`** (the default): every scope's registry in `registry` is the app's proxy, `compmd/registry/` under the app's base, so the registry learns nothing about readers (R1, F1). The dev server and `vite preview` serve it, with `registryProxy` from `@compmd/registry`; a deployed app serves it the same way, from a worker or its server (`registryProxy({ registries, base: "/compmd/registry/" })`, with the same registries). A static host with no server uses `direct` instead.
- **`fetch: "direct"`**: `registry` holds the registries as named, and the reader's browser fetches from them (R1, F2), which the host tells its readers. There is no proxy.
- **The size cap** is `maxPackSize`, when given; `resolvePacks`'s 10 MB otherwise (R6).
- **Built modules from a registry** come as code (`{ code }`, R2): `resolved.modules` gives them, and `sandbox.modules` the installed packs'.
- The app installs `@compmd/registry`, which `virtual:compmd` imports for `registry`.

## Questions, with proposed answers

**V1. Where a release's integrity comes from.** The processor compares an import's `integrity` with the release's (spec §6.4). In a build, packs come from `node_modules`, which npm checked against the lock file when it installed them. Proposed: the plugin reads each pack's `integrity` from the lock file (`pnpm-lock.yaml`, `package-lock.json`; Yarn later), and gives none when there is no lock file, so a document's `integrity` key then fails to match, which the processor reports (F035). In `pnpm-lock.yaml` it reads the packages of every YAML document: before the project's lock, pnpm may write its settings, or, for a project that pins its toolchain in `devEngines`, an environment lock with pnpm's and Node's packages. When there is a lock file but it has no integrity for a pack installed from a registry (its folder in `node_modules`, not a folder pack or a linked workspace package), the build warns, since that pack's pinned imports would fail with nothing else to say why. The alternative, hashing the installed files ourselves, gives a hash no registry publishes, so no document could write it.

**V2. A document importing a pack that isn't installed.** With P2 a document may import any pack, but a build has only what's installed. Proposed: such an import is F026, as a pack the registry doesn't have ("not installed in this app"); the plugin never fetches from the registry at run time. Fetching would make the app depend on the registry being up, and tell it who reads what (spec §15, privacy). *Since [packs from a registry](packs-from-a-registry.md):* this holds for an app whose configuration names no registry; one that names registries fetches such a pack at run time, through its own proxy unless it says `direct` (decision 8).

**V3. A trusted pack's source in `request`.** The processor reads a component file a pack imports to know it exists and to check it (W104). Proposed: the request at run time carries a trusted pack's component files as empty text, since the app's bundle already holds their code and their checks ran at build; to be confirmed against the processor's loading.

**V4. Which Vite.** Proposed: Vite 8 and later, as a peer dependency, as `@compmd/sandbox`'s build needs (its `buildLibraries` uses Vite 8's bundler); the plugin uses only `configResolved`, `buildStart`, `resolveId`/`load` for the virtual module, `configureServer` and `generateBundle`. Other bundlers later, with the same configuration.

**V5. Where `policyOf` and the schema live.** The playground reads the configuration without Vite. Proposed: in `@compmd/vite` as a Node-free entry, `@compmd/vite/config` (`readConfig`, `policyOf`, the schema), so a host that isn't a Vite app imports it alone. A separate package would hold one function and a schema.

## The example and the tests

- **`vite/example/`:** a small Vite app with `components/` as its app pack (`stars` in React, trusted and ambient), a document that uses `stars` and a code block of a strict pack from a folder installed as a dependency, rendered in host mode.
- **Unit tests (Node):** reading and translating the configuration, with every error; finding packs; reading integrity from both lock files; the size limit.
- **Registry tests:** an app with registries on the dev server and in a build under `vite preview`, against a local registry: with `host`, the proxy passes metadata and tarballs on and the page's registries are the proxy; with `direct`, they are the registries named, and there is no proxy; without registries, `registry` is `undefined`.
- **Build tests:** the example built with the plugin; the output has the frames under their hash (`compmd/frames/<hash>/strict.html`), the pack files at their places with the hashes `modules` gives, `_headers`, no manifest, and the virtual module's contents; `buildBundle` writes the layout and the manifest `compmd-host` reads.
- **Browser tests (Vitest browser, Chrome and WebKit):** the example's page: `stars` mounts in the page and sets its input; the strict pack's component runs in a frame; a denied pack's use shows its fallback with its reason.
- **Published** with the other packages (`RELEASING.md`, the release workflow).

## Not in this change

- Other bundlers (Rollup alone, webpack, esbuild).
- Libraries from the registry for packs (S3, still open): `libraries` is the host offering libraries, as the playground offers d3, not packs naming theirs.
- The playground's own use of the configuration.
