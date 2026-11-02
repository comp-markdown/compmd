# Packs from a registry

**Status: accepted (2026-10-31).**

A document imports `@kit/mermaid@1`, and the host has never heard of it. The accepted design for [components from apps and packs](components-from-apps-and-packs.md) says a document may import any npm pack, which runs in the strict sandbox (P2), and that a host may fill its packs from the registry at run time (H2). Neither is built: `@compmd/vite` gives the processor the packs an app installed at build time (H3) and its folder packs (H4), so an import of anything else is F026. This design is how a host lets its documents use packs straight from a registry, and what a CompMD pack on npm must be.

The spec settles most of what a pack on npm is (§6.4): `@scope/pack` is the npm package of that name, from npm or a registry the processor is configured with; the `compmd` field of its `package.json` names its document, by default `index.comp.md`; `package.json` is the manifest `export.pack` and `export.version` must match (F022); an import's `integrity` is the hash the registry publishes; `@compmd/*` is reserved and never fetched; names are scoped (F026 otherwise); a processor may cap a pack's size; a built module loads by its place in the pack, from files checked against the pack's integrity (F035). This design builds on those and decides only what they leave open.

## Proposed decisions

1. **A registry is something the host gives the processor, never something a document names.** A new package, `@compmd/registry`, defines a `Registry` (list a pack's versions; get one version's files, checked) and implements `NpmRegistry` for npm and any registry that speaks its protocol (a mirror, a private registry, a proxy). A host that is fine with it gives its documents registries in its configuration, per scope, as `.npmrc` does; without one, an import of a pack the host doesn't have stays F026, as today.
2. **Resolving packs is a step before processing, like fetching data** (R4). `sources(request)` already tells a host which remote and `host:` data a document needs; it also lists the packs the document and its packs import that the request doesn't hold. A resolver in `@compmd/registry` loops: ask `sources`, resolve each range against the registry's versions, fetch, add the release to the request, until nothing is missing. The processor stays synchronous and fetches nothing.
3. **A pack is checked against the hash npm publishes, the one an import's `integrity` holds** (R3). The resolver downloads the version's tarball, checks it against `dist.integrity` (and against the import's `integrity`, when the document pins one), and unpacks it in memory. Nothing is trusted that the tarball's hash doesn't cover.
4. **A registry pack's built module reaches its frame as text, already checked** (R2). The host's page holds the checked files, so it sends a built module to the frame as it sends compiled source, as a `Blob` whose bytes the frames share, and the frame loads it as a `blob:` module; what that costs is in R2. The strict frame's policy doesn't change: it still loads nothing from the network.
5. **Who fetches is the host's choice, said in its configuration; the reader's browser fetches directly only when the host says so** (R1). By default a host points its registries at its own proxy or cache, so the registry learns nothing about readers, as §6.4 requires. A host with no server (a static site, the playground) may choose `direct`: the reader's browser fetches from the registry, which then learns the reader's address and the names of the packs, never the document. The spec's privacy rule gains that exception, opt-in and stated.
6. **A registry pack is a CompMD pack as §6.4 says, plus three rules** (R6): it says it is one, in a typed field and for search (the `compmd` field of its `package.json`, written even when it names the default document, and the keyword `compmd-pack`, which only packs use); each built module is one file, with no chunks of its own; and its size is under the host's cap, which the registry's metadata states before anything is downloaded. Its npm `dependencies` are never installed: a pack bundles what it uses, but the runtimes the frames give and the libraries it shares (decision 8).
7. **Documents work while packs load, as the spec already allows.** A processor may render with the packs it has: a pack it doesn't hold yet renders as the spec says for any pack it doesn't hold (its uses show their fallback text, F026), and when the pack arrives the document renders again, as §8.5 allows when an input of rendering changes. The spec has no state for "on its way" (all loading is stage 4a, §11), so a reader can't yet tell a pack that is loading from one that won't load; that is its own design, [handling pack loading states](handling-pack-loading-states.md).
8. **Packs can share a library through the registry** (R9). A library published as one module file, and marked as a library pack, is fetched and checked like a pack; packs that name it in `peerDependencies` share one copy per document, the newest version every pack's range allows. A library that can't have two copies in a frame says so, and a second version of it is refused.

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

A strict frame loads scripts only from the frames' origin, and `blob:` modules ([the sandbox](running-components-in-the-sandbox.md), Q3). A registry pack's built module is elsewhere.

| | O1. The host page sends it as text (recommended) | O2. The frame's policy admits the registry's files | O3. The host serves it |
|---|---|---|---|
| How | the resolver has the checked files in the host page; the sandbox sends a built module to the frame as it sends compiled source, and the frame makes a `blob:` module | the policy adds `script-src https://cdn.jsdelivr.net/npm/@kit/mermaid@1.0.0/`, per pack, and the module loads with its file's hash | a proxy (F1) serves pack files from the host's origin, under `compmd/`, as `@compmd/vite` serves installed packs |
| Checked against | the tarball's hash, before it is sent | jsDelivr's per-file hash list, which npm doesn't sign | the tarball's hash, by the proxy |
| Network in the strict frame | none, as today | requests to the CDN, for that pack's files: a query string (`?leak=…`) on an allowed path reaches the CDN's logs | none beyond the host's own origin, as today |
| Needs | a built module to be one file: a `blob:` module's relative imports have nothing to resolve against | per-pack policies, so a frame per pack | a server |
| Caching | the resolver's own (R4): by integrity, in memory and in Cache Storage | the browser's, by URL, in WebKit; none in Chrome, which caches nothing a strict frame loads (below) | the same as O2 |

O1 changes nothing in the frames and keeps rule 2 (no network) exactly as it is; it works with F1 and F2. Its one requirement, one file per module, is decision 6. O3 remains for a host that runs a proxy and prefers URLs, and needs nothing new: its proxy's files are the host's.

**The sandbox's side:** `sandbox.modules(place)` today returns `{ url, integrity }`; it may also return `{ code }`, a `Blob` the host has checked, which the sandbox sends as it sends compiled source, with the plain binding (spec §7.6).

**What it costs.** The text isn't copied per frame: a `Blob` posted to a frame shares its bytes with the host's page, and the frame makes its own `blob:` URL from it (a `blob:` URL is its maker's origin's only, so the host's page can't make it for the frame). What every frame pays is parsing and compiling the module, as with a URL. A URL could save compiling again where the browser keeps compiled code for it, and downloading again where it caches the file. What browsers do for a strict frame (measured 2026-11-02):

- **Chrome caches nothing a strict frame loads:** no HTTP cache, no code cache. A module loaded by URL is downloaded again for every frame, every frame an editor rebuilds and every visit, on a laptop and on a phone. Within one process, V8 does reuse what it compiled for the same URL, so a later frame compiles less: on an Apple M4, none of Mermaid's functions again, against 12 ms for a `blob:` module.
- **WebKit caches a strict frame's requests by URL**, across visits too. It keeps no compiled code to show: a `blob:` module and a URL were within 10 ms.
- **A frame's boot** (the sandbox's strict frame with a small React component, p95) is 33 ms in Chrome on an Apple M4, 80 ms in WebKit there, and 253 ms in Chrome on a Galaxy Z Fold5.

Mermaid as one module file, minified (3.6 MB, 0.8 MB gzipped), its import and first diagram, p95 in ms:

| Case | Chrome, laptop: URL / `blob:` | WebKit, laptop | Chrome, phone (Wi-Fi) |
|---|---|---|---|
| One frame | 171 / 93 | 74 / 66 | 510 / 196 |
| Two frames in host mode: the second | 137 / 157 | 124 / 167 | 639 / 353 |
| An editor rebuilding a widget's frame as it scrolls ([the sandbox](running-components-in-the-sandbox.md), Q6): later builds | 61 / 97 | 58 / 68 | 561 / 192 |
| A visit after the first | 61 / 95 | 83 / 83 | 519 / 220 |

So O1's extra cost is compiling again, where a URL would have the compiled code in the same process: 20 to 40 ms a frame on a laptop, for a large module. A URL costs a download per frame in Chrome. On the laptop the server was local, so that download cost nothing. On the phone it made a URL 1.8 to 3.3 times slower than O1. A small module (ten charts, 50 KB each) costs little either way.

**Making it cheaper.** What compiling costs depends on what we can avoid: compiling the same code in many frames, compiling code that never runs, and compiling again what was compiled before.

| Way | What it saves | What it needs |
|---|---|---|
| **A service worker serving the checked files as URLs** at the frames' origin, so a static host gets URLs without a server | it would have given the code cache and chunks for free | **not possible:** a frame with an opaque origin has no service worker, so its requests go to the network (checked 2026-10-31: the Service Worker spec's issue 1390 says no major browser hands them to one, and Chromium stopped doing it on purpose; measured 2026-11-02 in Chrome and WebKit: a worker at the frames' origin serves a frame of that origin, never a strict one) |
| **Fewer frames that load the module** | one compile per frame that runs it | frame mode (one frame for the page), or keeping frames alive while an editor scrolls (`moveBefore`, [the sandbox](running-components-in-the-sandbox.md), Q6); host mode with ten diagrams stays ten compiles |
| **Compile only what a document uses** | Mermaid's chunks for the diagram drawn, not all 2.7 MB | chunks (R7) |
| **The code cache** | compiling again on later visits and in later frames | **not there for a strict frame:** Chrome keeps no code cache for it, and WebKit shows none (above), so O3's URLs save no compile |
| **Less than it looks** | browsers compile a function only when it first runs; a module's full cost is parsing it and compiling what it runs | measuring it |

**Recommendation: O1** (measured 2026-11-02, above). Where it costs more than a URL, in a second frame of a module and in a frame an editor rebuilds, frame mode and frames kept alive remove the extra compile. Chunks and O3 aren't needed for it: O3's URLs are downloaded again for every frame in Chrome.

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
- **The cache** keeps what the resolver fetched, by what can't change:

  | What | Kept by | Where | For how long | Not kept when |
  |---|---|---|---|---|
  | A version's files (the tarball, checked and unpacked: the documents and the files they import, not the rest) | its integrity | memory, for the page's life; Cache Storage, across visits and offline | until the host's cache is cleared: a version's tarball never changes | there is no Cache Storage: a page with an opaque origin (an application's sandboxed editor, [the sandbox](running-components-in-the-sandbox.md), decision 2), a private window that refuses storage, or a host that turns it off; then memory only |
  | A pack's versions list (to resolve a range) | the registry's URL and the name | memory and Cache Storage | the registry's `max-age` (npm's is 300 seconds); offline, a stale list is used rather than none | — |
  | Nothing is fetched at all | — | — | — | an import with an exact version and an `integrity` whose files are in the cache |

  The cache belongs to the host's page and its origin: two sites never share one, and a frame never sees it. A version is found by its integrity, so a cache can't serve a pack that doesn't match. A host may give its own cache (`cache` in the options), for example one on its server behind F1.
- **An editor** processes on every keystroke: it calls `process` at once with the packs it has, and `resolvePacks` in the background when `sources` names a pack it doesn't have; when it arrives, the editor processes again (decision 7). `OpenDocument` takes new packs the same way it takes new data.
- **`@compmd/vite`**: the host app's configuration gains `registries` and `fetch` (R5); `virtual:compmd` then gives a ready `registry` beside `request` and `sandbox`, and in dev the plugin's server is a proxy (F1), so development doesn't need `direct`.

## R5. The configuration

This is the host's configuration, never a pack's or a document's: `compmd.config.json` belongs to the application that renders documents (it is what `@compmd/vite` reads, beside the app's `package.json`), as trust levels, `imports` and `deny` already do. A pack's `package.json` says nothing about registries, and a document only names packs. The packs a pack imports are fetched through the same host's registries, under the same policy.

**An example.** A site, `notes.example`, renders its users' documents, and has a private registry for its own packs:

```json
{
  "registries": { "@notes": "https://npm.notes.example/", "*": "/compmd/registry/" },
  "fetch": "host",
  "imports": "any",
  "packs": { "@notes/widgets": { "trust": "permissive", "network": ["https://api.notes.example"] } }
}
```

A user's document imports `@kit/mermaid@1` and `@notes/widgets@2`. The resolver asks the site's own registry for `@notes/widgets`, and the site's proxy at `/compmd/registry/` (which passes requests on to npm) for `@kit/mermaid`. `@kit/mermaid`'s document imports `@kit/theme@1`: that is fetched through the proxy too, since it is the site's configuration that decides, not `@kit/mermaid`. `@notes/widgets` runs permissive, reaching the site's API, because the site listed it; `@kit/mermaid` and `@kit/theme` run strict, as every unlisted pack does. If a document imports `@evil/pack` from a scope the site never listed, it comes from `*`, under the site's policy (strict, its size capped, `deny` applied). A document can't name `https://evil.example/` as a registry, and neither can a pack. Another host rendering the same document, say the playground, uses its own configuration: `"*": "https://registry.npmjs.org/"` with `"fetch": "direct"`.

The keys:

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

- **It says it is a pack, twice.** npm's search indexes only a package's name, description and keywords, never its other `package.json` fields, so a typed field can't be searched for, and a keyword can't be typed:
  - **the `compmd` field**, written even when it names the default (`"compmd": "index.comp.md"`), is the typed marker: npm keeps every `package.json` field in a version's manifest (`GET <registry>/<name>/<version>`, checked 2026-10-31 with a package's custom `svelte` field), so a tool can confirm a search result is a pack without its tarball. A resolver still reads a package without it as §6.4 says.
  - **the keyword `compmd-pack`**, so `GET <registry>/-/v1/search?text=keywords:compmd-pack` lists them. Only packs use it; tools that help build packs (`@compmd/create-pack`, the Vite plugin) use `compmd` instead, so a search for packs finds packs. A search tool keeps only the results whose manifest has the field.
  The resolver needs neither: an import names its pack.
- **No required name prefix.** npm's prefixes (`eslint-plugin-x`, `babel-plugin-x`, `prettier-plugin-x`, `vite-plugin-x`, `remark-x`) exist mostly so a tool can expand a short name (ESLint turns `x` into `eslint-plugin-x`) and so unscoped names don't collide. A CompMD import is always written in full and always scoped, so a prefix would only make names longer (`@kit/compmd-pack-mermaid`); the field and the keyword already say what it is. A scope that publishes both a pack and a library of the same name MAY write `compmd-` (`@acme/charts`, the React library, beside `@acme/compmd-charts`, the pack).
- **Each built module is one file**, with no relative imports (R2, O1); its only imports are the runtimes the frames give (React, React DOM, Svelte). `npm create @compmd/pack` already builds one file per component; it should also turn off code splitting, so a library that loads parts of itself on demand (Mermaid does) stays in the one file. What that costs, and what chunks would need, is R7.
- **Libraries are bundled in each pack** that uses them, so ten chart packs that use d3 bring ten copies of the parts of d3 each uses: ten downloads, cached by each pack's integrity, and ten compiles in a frame that runs all ten. d3 is about 280 KB minified in full, and a chart pack's bundler keeps only the modules it imports (scales, shapes, axes: tens of KB), so ten such packs cost hundreds of KB, not megabytes. The cases where sharing pays are large libraries that several packs use (three.js, KaTeX, a chart library used whole). Sharing them needs one copy loaded once per frame and found by name: the import map's job, which today is fixed when the frames are built (its hash is in the policy), so a library from the registry can't join it at run time. R9 is how packs share them.
- **Its npm `dependencies` are never installed** by a host that fetches it: what a pack's components use is bundled into their modules, and its own imports of other packs go through `md.import`, which the resolver follows. `dependencies` may still list what the build used.
- **A size cap**, the host's `maxPackSize`, applied to the version's `unpackedSize` in the metadata before anything is downloaded. Today the key has no default; for packs from a registry, 5 MB when the host sets none, enough for Mermaid's module with room to spare.
- **`files` in `package.json`** should leave out what a host never reads (sources, tests), so the tarball is small; the template sets it.

## R7. One file per module, and what chunks would need

**What one file costs.** Mermaid 11 (checked 2026-10-31, `mermaid@11.12.0` on jsDelivr) ships two ways: one file of 2.7 MB minified, or an entry of 26 KB that loads 73 chunks (2.7 MB together) as a diagram needs them, its parser, its layout, KaTeX for math. A document that draws one flowchart loads only some of the chunks; with one file, it loads all 2.7 MB (about 0.8 MB gzipped), once per version thanks to the cache, and compiles it in each frame that draws a diagram. For most packs (a few charts, a widget) one file is what their bundler makes anyway; the cost is for large libraries that split themselves.

**What chunks need with O1.** A chunk is a module that imports others by relative path (`./chunk-7SRKK4IT.mjs`), and a `blob:` module has no path to resolve them against. Three ways:

| | K1. Rewrite the imports into `blob:` URLs | K2. A module loader in the frame | K3. The pack's files by URL (O3) |
|---|---|---|---|
| How | the host's page (or the frame) parses each file's imports and writes each relative one as the `blob:` URL of that file, leaves first | a loader such as es-module-shims, given the pack's files, which rewrites imports as K1 does and handles cycles with stand-in modules | the host's proxy serves the pack's files from its origin, and the browser loads chunks as any module's imports, each checked by its hash (M1) |
| Needs | an import parser (es-module-lexer, about 4 KB) and dynamic `import()` with literal paths; **fails on chunks that import each other**, which bundlers make: a `blob:` URL must exist before the text that names it | the loader in every frame (about 20 KB); its fast parser is WebAssembly, which the strict policy forbids without `'wasm-unsafe-eval'`, so its slower parser; a spike, to check it in our frames | a server (F1); nothing in the frames |
| Loads | only the chunks a diagram needs, each checked when the tarball is | the same | the same |
| Risk | text rewriting of code that was checked: a bug changes which code runs, of that pack only | the loader's, in every frame | none new |

**Recommendation:** one file per module now, which every host can load; chunks later, by K3 for hosts with a proxy (no new code in the frames) and by K2 for hosts without one, once a spike shows it works under the strict policy. Neither changes the pack format: a pack that ships chunks is still a valid pack, and a host that can't load them shows its components' fallbacks, with a diagnostic. The template's one-file build is what keeps today's packs loadable everywhere.

## R8. A realistic example: what it costs

A team's quarterly report, read on a static site with `"fetch": "direct"`, in host mode:

- it imports `@acme/report-kit@2`: containers written in CompMD (a 20 KB tarball), which itself imports `@acme/charts@3` (five Svelte charts, built modules with their parts of d3, 150 KB unpacked) and `@kit/mermaid@1` (one 3.6 MB module, a 0.8 MB tarball);
- the report uses five charts and two diagrams.

**Fetching**, the first visit. Pack imports form a tree, and each level waits for the one above it, since only the parent's document says what it imports; packs on one level are fetched together.

| Round | Requests | About (100 ms to npm, 20 Mbit/s) |
|---|---|---|
| 1 | `report-kit`: versions, tarball | 0.2 s |
| 2 | `charts` and `mermaid` together: versions, tarballs | 0.2 s, plus 0.4 s for Mermaid's tarball |
| Then | nothing: no further imports | |

About 0.8 s until every pack is there. The text, the tables and the CompMD containers render at once; the charts and diagrams show their fallback text (decision 7) until their packs arrive. An import with an exact version and `integrity` skips its versions request (npm's tarball URL follows from the name and version); a host's proxy (F1) could resolve a whole tree in one request, a later addition. On a later visit, nothing is fetched but expired versions lists (R4).

**Frames.** A pack's imports don't make frames: frames follow trust ([the sandbox](running-components-in-the-sandbox.md), decision 1), so they are at most three deep (the host's page, a permissive frame, a strict frame), whatever the depth of the pack tree. Here, in host mode: seven strict frames (five charts, two diagrams) and the compile frame; in frame mode, one page frame and the compile frame.

**Memory and compiling**: Mermaid's measured (2026-11-02, R2), the rest estimated, to be measured when built ([the sandbox](running-components-in-the-sandbox.md), Q9):

| | Host mode | Frame mode |
|---|---|---|
| Pack files in the host's page | 3 MB of `Blob`s, shared with the frames | the same |
| Frames | eight, each a few MB for its document, the sandbox's runtime and Svelte's | two |
| Mermaid | compiled twice (once per diagram's frame): its import and first diagram about 100 ms each on a laptop (Apple M4) and 200 ms on a phone (Galaxy Z Fold5), the second more while both load at once; about 20 MB of heap each once it draws | compiled once |
| Charts | five small compiles | in the page's frame |

So what grows with a document is frames, which the host chooses ([the sandbox](running-components-in-the-sandbox.md), Q1), not the depth of its packs; a large module costs a compile per frame that uses it, which frame mode keeps down; no browser keeps compiled code for a strict frame across visits (R2).

## R9. Libraries packs share (S3)

**Today** a pack bundles every library it uses, but the runtimes, which every frame shares through its import map (one React, so hooks work). Ten chart packs that use d3 bring ten copies of the parts they use: cheap for d3 (tens of KB each, R6), not for large libraries several packs use whole (three.js, about 700 KB minified; KaTeX, 270 KB).

**Do different versions conflict?** Two copies of a library, of one version or two, are separate modules, so they don't see each other. They meet only in one frame (in frame mode, the page's frame holds every strict use; in host mode, a use and the uses in its slots), and they conflict only through what a browser keeps once per frame:

| What | Example | With two copies in one frame |
|---|---|---|
| Custom elements | a pack built with Lit or Shoelace defines `<sl-button>` | the second `customElements.define` throws: the second pack's component fails (W118) |
| Global styles | a library injects CSS for its own classes | the later copy's rules win for both |
| Globals and patched prototypes | a UMD build sets `window.d3`; a polyfill patches `Array.prototype` | the last one wins |
| One instance by design | React, a store, an i18n singleton | two states that don't know each other: why the runtimes are shared |
| Objects passed between packs | a d3 scale handed from one pack to another, checked with `instanceof` | can't happen: params are values (§8) and slots are rendered content, so packs never pass each other library objects |

So most libraries mix versions safely; the ones that don't are those with global effects. Bundling has these conflicts today; sharing can remove them, by loading one copy.

**Ways to share:**

| | L1. Libraries the host offers (today) | L2. Library packs from the registry (recommended) | L3. A CDN that builds npm packages into modules (esm.sh) |
|---|---|---|---|
| How | the host's configuration lists `libraries`; they are in the frames' import map, built with the app; a pack leaves them out of its build | a library is published as one module file and says so (`"compmd": { "library": "dist/index.js" }`, for example `@kit/d3`); a pack names the ones it shares in `peerDependencies` (`"@kit/d3": "^7.9"`) and imports them by that name; the resolver gets them as it gets packs, and the frame loads each as a `blob:` module, with the pack's imports of it written as that module's URL | the host's page imports `https://esm.sh/d3@7` |
| Any library at run time | no: only what the host built in | yes, once someone publishes it as a library pack (the pack's author can) | yes, by its npm name |
| Checked against | the app's lock file | the library pack's tarball, like any pack (R3) | nothing npm published: esm.sh builds the file |
| A pack that needs one, on a host without it | fails (its components' fallbacks) | works: the resolver gets it | works |
| New in the frames | nothing | an import parser (es-module-lexer's plain JavaScript build, about 4 KB, since the strict policy has no WebAssembly) to write a library's name as its URL; no cycles, since a library imports only other libraries | network to the CDN |

**L2's rules:**
- **One version per document:** for each library, the newest version that satisfies every range the document's packs ask for, so a frame has one copy. When no version satisfies them all, each range gets its own; a library that can't have two copies in a frame says so (`"compmd": { "library": …, "single": true }`), and then the pack asking for the second gets W121 on its import, naming the library and both ranges, as for any `peerDependencies` the processor can't meet (spec §6.4): its components that don't use the library still render.
- **A library runs at the level of the pack that imports it,** in its frame; it adds no trust of its own, and a library pack has no document and no components.
- **The policy applies as to packs:** `deny`, the size cap, `imports: listed`.
- **What the frame trusts:** the host's page writes the library's URL into the pack's text after checking both, as K1 would (R7); a library's own imports are only library names, so there is no relative path to write.

**Why not any npm package as it is published?** It would be simpler for authors if a library were just the library's own package, `d3` itself. What stops it is what a frame can load: one checked module file, with no imports but names it knows. Published packages are rarely that. d3 7.9 (checked 2026-10-31) has its module entry at `src/index.js`, which imports 30 other packages (`d3-array`, `d3-scale`, …), each of many files, some with dependencies of their own; its one-file build, `dist/d3.min.js`, is a classic script that sets `window.d3`, not a module. Other packages ship only CommonJS, which a browser can't import at all.

| | A library pack (recommended) | Any npm package, its files as published | Any npm package, bundled by the host |
|---|---|---|---|
| What is fetched | one tarball with one module | the package and its whole dependency tree: for d3, more than 31 packages (it and its 30, then theirs), each a versions request and a tarball on a first visit | the same tree, on the host's server |
| Loading it in a frame | as one `blob:` module | a module loader resolving Node's rules (`exports`, `module`, bare imports between the packages) over hundreds of files: K2 (R7), with more | one module, from the host's proxy |
| CommonJS packages | the library pack's build converts them | refused | the bundler converts them |
| Static hosts | yes | yes, once K2 works | no: needs a server (F1) |
| Who does the work | whoever publishes the library pack, once | every host's page, on every first visit | every host with a server |

So the two are the same kind of thing, an npm package fetched and checked the same way, and differ in one field and in being built for the browser. A library pack is the library's own code, built: its name says what it wraps and its version follows the library's (`@kit/d3@7.9.0` is `d3@7.9.0`), and anyone can publish one, the library's own authors included. Any npm package as published becomes possible once K2 is, through the same loader; it is later work, not a different design.

**It meets the spec where its rules for a component's imports are open:** what a bare import in a component may resolve to. With L2, the answer would be the binding's runtime, a library the host offers (L1), or a library the pack names in `peerDependencies` (L2); anything else is its fallback with a diagnostic. Those rules are settled first; L2 builds on them.

## R10. What changes

**A companion to the spec:** *CompMD packs on a registry* (`compmd-registry-v1.md`, next to the spec), the normative part of this design for pack authors and registry hosts: what R6 says a registry pack is (the field, the keyword, names, one file per module, `files`, the size cap and how it is read), how a host reads one (R3), and what a library pack is (R9). The spec's §6.4 links to it, as it stays about documents; it is versioned with the spec.

**The spec:** §6.4 *The processor gets packs*: the exception for a host that lets the reader's browser fetch from a registry its configuration names (R1); §15 *Privacy*: the same, with what it discloses; §6.4 *Built modules*: a built module SHOULD be one file, and MUST be for a host that loads it as text; F026's row keeps its cases. Telling a loading pack from one that can't be had is [its own design](handling-pack-loading-states.md).

**The processor:** `sources` lists the packs a request doesn't hold, from the document and from the packs it holds, with ranges and pinned integrity.

**The packages:**
- `@compmd/registry`, new: `Registry`, `NpmRegistry`, `resolvePacks`, the tarball reader (gzip, tar, integrity), the cache, and a proxy handler for Node and workers; tests against a local registry, and in Chrome and WebKit. How it does each is its own design: [resolving registry packs](resolving-registry-packs.md).
- `@compmd/sandbox`: `modules(place)` may give `{ code }` (R2); library packs loaded as `blob:` modules, and a pack's imports of them written as their URLs (R9).
- `@compmd/registry`, for R9: library packs resolved with packs, one version per document, `single`.
- `@compmd/vite`: `registries`, `fetch` and the size cap in the configuration and its schema; `registry` in `virtual:compmd`; the dev server as a proxy.
- `@compmd/create-pack`: the keyword, one file per module with no code splitting, `files`; shared libraries as `peerDependencies`, left out of the build; a template for a library pack.

**The playground:** registries with `direct`, said in its *About*, so a document can import any published pack; a document that does.

**The guides:** the Author's Guide's *Using a pack* says how a document imports a pack from npm; the Developer's Guide gains registries in *Packs from npm* and the conventions in *Testing and publishing*.

## Not in this change

- Checking npm's signatures or provenance (`dist.signatures`, attestations): the tarball's hash is checked; signatures can follow.
- Registries other than npm's protocol. Nearly every registry a host would use speaks it: Verdaccio, GitHub Packages, GitLab, Artifactory, Nexus, AWS CodeArtifact (those that need a token only behind the host's proxy, F1, never with `direct`), and JSR through its npm endpoint (`https://npm.jsr.io/`, which answers cross-origin requests, checked 2026-10-31), where `@std/path` is named `@jsr/std__path`. A registry that doesn't (a CDN such as esm.sh) is a file server, not a registry: F3.
- Pack chunks (R7): one file per module first.
- Searching for packs from an editor: one request (`GET <registry>/-/v1/search?text=keywords:compmd-pack+<words>&size=20`, which npm answers cross-origin, cached for 300 seconds, with each package's name, version, description and links), a check of each result's `compmd` field (R6), and a picker that writes the `md.import` entry with its range and `integrity`. Showing a pack's components before importing it means resolving it. A later change; private registries may have no search (GitHub Packages has none).
