# Packs from a registry

**Status: proposed (2026-10-31).**

A document imports `@kit/mermaid@1`, and the host has never heard of it. The accepted design for [components from apps and packs](components-from-apps-and-packs.md) says a document may import any npm pack, which runs in the strict sandbox (P2), and that a host may fill its packs from the registry at run time (H2). Neither is built: `@compmd/vite` gives the processor the packs an app installed at build time (H3) and its folder packs (H4), so an import of anything else is F026. This design is how a host lets its documents use packs straight from a registry, and what a CompMD pack on npm must be.

The spec settles most of what a pack on npm is (§6.4): `@scope/pack` is the npm package of that name, from npm or a registry the processor is configured with; the `compmd` field of its `package.json` names its document, by default `index.comp.md`; `package.json` is the manifest `export.pack` and `export.version` must match (F022); an import's `integrity` is the hash the registry publishes; `@compmd/*` is reserved and never fetched; names are scoped (F026 otherwise); a processor may cap a pack's size; a built module loads by its place in the pack, from files checked against the pack's integrity (F035). This design builds on those and decides only what they leave open.

## Proposed decisions

1. **A registry is something the host gives the processor, never something a document names.** A new package, `@compmd/registry`, defines a `Registry` (list a pack's versions; get one version's files, checked) and implements `NpmRegistry` for npm and any registry that speaks its protocol (a mirror, a private registry, a proxy). A host that is fine with it gives its documents registries in its configuration, per scope, as `.npmrc` does; without one, an import of a pack the host doesn't have stays F026, as today.
2. **Resolving packs is a step before processing, like fetching data** (R4). `sources(request)` already tells a host which remote and `host:` data a document needs; it also lists the packs the document and its packs import that the request doesn't hold. A resolver in `@compmd/registry` loops: ask `sources`, resolve each range against the registry's versions, fetch, add the release to the request, until nothing is missing. The processor stays synchronous and fetches nothing.
3. **A pack is checked against the hash npm publishes, the one an import's `integrity` holds** (R3). The resolver downloads the version's tarball, checks it against `dist.integrity` (and against the import's `integrity`, when the document pins one), and unpacks it in memory. Nothing is trusted that the tarball's hash doesn't cover.
4. **A registry pack's built module reaches its frame as text, already checked** (R2). The host's page holds the checked files, so it sends a built module to the frame as it sends compiled source, and the frame loads it as a `blob:` module. The strict frame's policy doesn't change: it still loads nothing from the network.
5. **Who fetches is the host's choice, said in its configuration; the reader's browser fetches directly only when the host says so** (R1). By default a host points its registries at its own proxy or cache, so the registry learns nothing about readers, as §6.4 requires. A host with no server (a static site, the playground) may choose `direct`: the reader's browser fetches from the registry, which then learns the reader's address and the names of the packs, never the document. The spec's privacy rule gains that exception, opt-in and stated.
6. **A registry pack is a CompMD pack as §6.4 says, plus three rules** (R6): it says it is one (the keyword `compmd-pack`, for search); each built module is one file, with no chunks of its own; and its size is under the host's cap, which the registry's metadata states before anything is downloaded. Its npm `dependencies` are never installed: a pack bundles what it uses, but the runtimes the frames give.
7. **Documents work while packs load.** A processor MAY render before a pack has arrived: the pack's components render their fallbacks, without F026, and the document renders again when it arrives. F026 stays for a pack that can't be had: no such pack, no matching version, a hash that doesn't match, over the cap, denied.

## Today

- **`@compmd/vite`** reads `compmd.config.json` at build time, finds every installed scoped dependency that is a pack and every folder pack it lists, reads each pack's documents and the files they import, takes its integrity from the app's lock file, and gives `virtual:compmd`'s `request.packs` (each pack's releases: version, integrity, files) and `sandbox.modules` (a built module's place, `pack@version/path`, to its URL in the app and its file's hash).
- **The processor** (`process`, `OpenDocument`) takes packs only in the request. An import whose pack isn't there is F026 (*not provided*); an import whose range no given release matches, F026 too. `sources(request)` lists the remote and `host:` data a document needs, which the host fetches and gives in `remote` and `host`.
- **The sandbox** loads a built module by URL with `<script type="module" integrity>` (the file's own hash, from the host), and compiled source as a `blob:` module made in the frame from text the host sends. The strict frame's policy allows scripts from the frames' origin and `blob:` modules, and no network.
- **The policy** in the request: trust per pack, `imports` (`any` or `listed`), `deny` (name, range, reason), `refused` (a pack the host left out for its size).
- **npm, from a browser** (checked 2026-10-31): the registry's metadata (`GET https://registry.npmjs.org/@scope%2fname`, with `Accept: application/vnd.npm.install-v1+json` for the short form) and its tarballs both answer cross-origin requests (`Access-Control-Allow-Origin: *`). The short form gives every version with its tarball URL, `integrity` (sha512 of the tarball), file count and unpacked size, but no `package.json` fields beyond dependencies. A tarball is a gzipped tar of the package under `package/`; browsers unzip with `DecompressionStream("gzip")`, and tar is a simple format. jsDelivr serves single files of any npm version (`cdn.jsdelivr.net/npm/<name>@<version>/<path>`, cross-origin, cached for a year) and lists them with their sha256 (`data.jsdelivr.com/v1/packages/npm/<name>@<version>`).

## R1. Who fetches, and what the registry learns

§6.4: "A processor that renders for a reader fetches packs itself or through its own cache, so the registry doesn't learn who reads which document." A processor running on a server or in a desktop app does that already. A processor running in the reader's browser, with no server behind it, has no "itself" but the reader's browser.

| | F1. Only through the host's proxy or cache | F2. Direct, when the host says so (recommended, with F1 as the default) | F3. Through a public CDN |
|---|---|---|---|
| How | the host runs a proxy (a small handler `@compmd/registry` ships, for Node and for workers) or a mirror, and `NpmRegistry` points at it | `"fetch": "direct"` in the configuration: the reader's browser fetches from the registry | `NpmRegistry` points at jsDelivr or unpkg instead of npm |
| The registry learns | nothing about readers | the reader's address, the time, and the packs' names and versions; never the document or what it shows | the same as F2, told to the CDN instead |
| Static hosts (the playground) | can't, without deploying a proxy | can, and say so | can, and say so |
| Spec | as today | §6.4 and §15 *Privacy* gain: a host MAY let the reader's browser fetch packs from a registry its configuration names, and SHOULD tell its readers | as F2 |

What F2 discloses is the same as what an image in a document discloses about a page view today (§15 *Privacy*), and less than a data fetch, which can carry input values: a pack's name says which packs a document uses, not what it says. No credentials are sent (the resolver fetches with `credentials: "omit"`), and nothing the document holds goes into a registry URL: only names and versions from its imports. A document can never choose F2: only the host's configuration can.

F3 is F2 with another party; it buys nothing F2 lacks for packs, since the tarball is checked either way (R3).

## R2. How a registry pack's module reaches its frame

A strict frame loads scripts only from the frames' origin, and `blob:` modules (`docs/sandbox.md`, Q3). A registry pack's built module is elsewhere.

| | O1. The host page sends it as text (recommended) | O2. The frame's policy admits the registry's files | O3. The host serves it |
|---|---|---|---|
| How | the resolver has the checked files in the host page; the sandbox sends a built module to the frame as it sends compiled source, and the frame makes a `blob:` module | the policy adds `script-src https://cdn.jsdelivr.net/npm/@kit/mermaid@1.0.0/`, per pack, and the module loads with its file's hash | a proxy (F1) serves pack files from the host's origin, under `compmd/`, as `@compmd/vite` serves installed packs |
| Checked against | the tarball's hash, before it is sent | jsDelivr's per-file hash list, which npm doesn't sign | the tarball's hash, by the proxy |
| Network in the strict frame | none, as today | requests to the CDN, for that pack's files: a query string (`?leak=…`) on an allowed path reaches the CDN's logs | none beyond the host's own origin, as today |
| Needs | a built module to be one file: a `blob:` module's relative imports have nothing to resolve against | per-pack policies, so a frame per pack | a server |
| Caching | the resolver's own (R4): by integrity, in memory and in Cache Storage | the browser's, by URL | the browser's, by URL |

O1 changes nothing in the frames and keeps rule 2 (no network) exactly as it is; it works with F1 and F2. Its one requirement, one file per module, is decision 6. O3 remains for a host that runs a proxy and prefers URLs, and needs nothing new: its proxy's files are the host's.

**The sandbox's side:** `sandbox.modules(place)` today returns `{ url, integrity }`; it may also return `{ code }`, text the host has checked, which the sandbox sends as it sends compiled source, with the plain binding (spec §7.6).

## R3. What is checked

| | I1. The tarball against `dist.integrity` (recommended) | I2. Each file against a CDN's hashes |
|---|---|---|
| Matches §6.1 | yes: an import's `integrity` is the tarball's hash, the one npm publishes and the lock files hold | no: a per-file hash is another value |
| Download | the whole tarball, once per version (a README and sources too) | only the files the document needs |
| Trust | npm's metadata (or a pinned `integrity` in the document); npm also signs each version (`dist.signatures`), which a later version of the resolver may check | jsDelivr's |

With I1, a pack is its tarball: the resolver reads `package.json` from it (so it learns the `compmd` field without the long metadata), its document, and the files its documents import, as `@compmd/vite` does from `node_modules`. The tarball's size is known before it is fetched (R6), so the cap costs nothing.

## R4. The API

```ts
import { NpmRegistry, resolvePacks } from "@compmd/registry";

const registries = { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) };
const { packs, modules } = await resolvePacks(request, { registries, policy: request.policy, cache });
const processed = process({ ...request, packs: { ...request.packs, ...packs } });
```

- **`Registry`**: `versions(name)` gives each version with its integrity and sizes; `get(name, version, integrity)` gives its files, checked. `NpmRegistry` implements both for the npm protocol, at any URL.
- **`resolvePacks(request, options)`** loops on `sources(request)`, which gains `packs`: each import (from the document and from packs already in the request) whose pack the request doesn't hold, with its range and pinned `integrity`. A range resolves to the newest matching version, as §6.1 says; then policy (`deny`, `imports: listed`, the size cap) is applied before anything is downloaded; then the tarball is fetched and checked; then the pack's own imports are asked for, until none are missing. It gives the releases for `request.packs` and a `modules` function for the sandbox (R2).
- **The cache** keeps packs by `name@version` and integrity, in memory, and in Cache Storage where there is one, so a pack seen once is there offline and on the next visit. Versions lists are kept for a short time (npm's own `max-age` is 300 seconds).
- **An editor** processes on every keystroke: it calls `process` at once with the packs it has, and `resolvePacks` in the background when `sources` names a pack it doesn't have; when it arrives, the editor processes again (decision 7). `OpenDocument` takes new packs the same way it takes new data.
- **`@compmd/vite`**: the configuration gains `registries` and `fetch` (R5); `virtual:compmd` then gives a ready `registry` beside `request` and `sandbox`, and in dev the plugin's server is a proxy (F1), so development doesn't need `direct`.

## R5. The configuration

```json
{
  "registries": {
    "@kit": "https://npm.kit.example/",
    "*": "https://registry.npmjs.org/"
  },
  "fetch": "direct",
  "imports": "any",
  "deny": [{ "pack": "@bad/pack", "reason": "draws fake prompts" }],
  "maxPackSize": "5 MB"
}
```

- **`registries`**: by scope, with `*` for every other scope, as `.npmrc` does. Without the key, no registry: imports of packs the host doesn't have stay F026. A document never names a registry, so it can't make a reader's browser contact a server of its choosing.
- **`fetch`**: `"host"` (the default: the registry URLs are the host's proxy or mirror, F1) or `"direct"` (F2).
- **`imports`**, **`deny`** and **`maxPackSize`** are the keys the configuration already has; the cap now also applies before a download, from the registry's metadata (R6).

## R6. What a CompMD pack on npm is

Beyond §6.4:

- **The keyword `compmd-pack`** in `package.json`, so `npm search keywords:compmd-pack` and the registry's search API list packs. It says what the package is for; the resolver doesn't need it.
- **Each built module is one file**, with no relative imports (R2, O1); its only imports are the runtimes the frames give (React, React DOM, Svelte). `npm create @compmd/pack` already builds one file per component; it should also turn off code splitting, so a library that loads parts of itself on demand (Mermaid does) stays in the one file.
- **Its npm `dependencies` are never installed** by a host that fetches it: what a pack's components use is bundled into their modules, and its own imports of other packs go through `md.import`, which the resolver follows. `dependencies` may still list what the build used.
- **A size cap**, the host's `maxPackSize`, applied to the version's `unpackedSize` in the metadata before anything is downloaded. Today the key has no default; for packs from a registry, 5 MB when the host sets none, enough for Mermaid's module with room to spare.
- **`files` in `package.json`** should leave out what a host never reads (sources, tests), so the tarball is small; the template sets it.

## R7. What changes

**The spec:** §6.4 *The processor gets packs*: the exception for a host that lets the reader's browser fetch from a registry its configuration names (R1); §15 *Privacy*: the same, with what it discloses; §6.4 *Built modules*: a built module SHOULD be one file, and MUST be for a host that loads it as text; the rendering before packs arrive (decision 7), in §11 or §6.4. F026's row keeps its cases.

**The processor:** `sources` lists the packs a request doesn't hold, from the document and from the packs it holds, with ranges and pinned integrity.

**The packages:**
- `@compmd/registry`, new: `Registry`, `NpmRegistry`, `resolvePacks`, the tarball reader (gzip, tar, integrity), the cache, and a proxy handler for Node and workers; tests against a local registry, and in Chrome and WebKit.
- `@compmd/sandbox`: `modules(place)` may give `{ code }` (R2).
- `@compmd/vite`: `registries`, `fetch` and the size cap in the configuration and its schema; `registry` in `virtual:compmd`; the dev server as a proxy.
- `@compmd/create-pack`: the keyword, one file per module with no code splitting, `files`.

**The playground:** registries with `direct`, said in its *About*, so a document can import any published pack; a document that does.

**The guides:** the Author's Guide's *Using a pack* says how a document imports a pack from npm; the Developer's Guide gains registries in *Packs from npm* and the conventions in *Testing and publishing*.

## Not in this change

- Libraries a host resolves for packs from the registry (S3 in the earlier design): deferred until two published packs want the same large library.
- Checking npm's signatures or provenance (`dist.signatures`, attestations): the tarball's hash is checked; signatures can follow.
- Registries other than npm's protocol.
- Searching for packs from an editor.
