# Packs from a registry

**Status: proposed (2026-10-31).**

A document imports `@kit/mermaid@1`, and the host has never heard of it. The accepted design for [components from apps and packs](components-from-apps-and-packs.md) says a document may import any npm pack, which runs in the strict sandbox (P2), and that a host may fill its packs from the registry at run time (H2). Neither is built: `@compmd/vite` gives the processor the packs an app installed at build time (H3) and its folder packs (H4), so an import of anything else is F026. This design is how a host lets its documents use packs straight from a registry, and what a CompMD pack on npm must be.

The spec settles most of what a pack on npm is (§6.4): `@scope/pack` is the npm package of that name, from npm or a registry the processor is configured with; the `compmd` field of its `package.json` names its document, by default `index.comp.md`; `package.json` is the manifest `export.pack` and `export.version` must match (F022); an import's `integrity` is the hash the registry publishes; `@compmd/*` is reserved and never fetched; names are scoped (F026 otherwise); a processor may cap a pack's size; a built module loads by its place in the pack, from files checked against the pack's integrity (F035). This design builds on those and decides only what they leave open.

## Proposed decisions

1. **A registry is something the host gives the processor, never something a document names.** A new package, `@compmd/registry`, defines a `Registry` (list a pack's versions; get one version's files, checked) and implements `NpmRegistry` for npm and any registry that speaks its protocol (a mirror, a private registry, a proxy). A host that is fine with it gives its documents registries in its configuration, per scope, as `.npmrc` does; without one, an import of a pack the host doesn't have stays F026, as today.
2. **Resolving packs is a step before processing, like fetching data** (R4). `sources(request)` already tells a host which remote and `host:` data a document needs; it also lists the packs the document and its packs import that the request doesn't hold. A resolver in `@compmd/registry` loops: ask `sources`, resolve each range against the registry's versions, fetch, add the release to the request, until nothing is missing. The processor stays synchronous and fetches nothing.
3. **A pack is checked against the hash npm publishes, the one an import's `integrity` holds** (R3). The resolver downloads the version's tarball, checks it against `dist.integrity` (and against the import's `integrity`, when the document pins one), and unpacks it in memory. Nothing is trusted that the tarball's hash doesn't cover.
4. **A registry pack's built module reaches its frame as text, already checked** (R2). The host's page holds the checked files, so it sends a built module to the frame as it sends compiled source, as a `Blob` whose bytes the frames share, and the frame loads it as a `blob:` module; what that costs is in R2. The strict frame's policy doesn't change: it still loads nothing from the network.
5. **Who fetches is the host's choice, said in its configuration; the reader's browser fetches directly only when the host says so** (R1). By default a host points its registries at its own proxy or cache, so the registry learns nothing about readers, as §6.4 requires. A host with no server (a static site, the playground) may choose `direct`: the reader's browser fetches from the registry, which then learns the reader's address and the names of the packs, never the document. The spec's privacy rule gains that exception, opt-in and stated.
6. **A registry pack is a CompMD pack as §6.4 says, plus three rules** (R6): it says it is one, in a typed field and for search (the `compmd` field of its `package.json`, written even when it names the default document, and the keyword `compmd-pack`, which only packs use); each built module is one file, with no chunks of its own; and its size is under the host's cap, which the registry's metadata states before anything is downloaded. Its npm `dependencies` are never installed: a pack bundles what it uses, but the runtimes the frames give.
7. **Documents work while packs load, and say so.** The spec has no such state today: all loading happens in stage 4a, before anything is evaluated (§11), so a pack is there or it is F026. A processor MAY now render before a pack has arrived: a use of one of its components is **pending**, marked in the expanded tree, and renders its fallback with `aria-busy="true"`, which a host may style (a placeholder, a spinner) and assistive technology announces. A pending use has no diagnostic: nothing is wrong yet. When the pack arrives, the document renders again; when it can't be had (no such pack, no matching version, a hash that doesn't match, over the cap, denied, the fetch failed), the use renders its plain fallback with F026, as today. So a reader and a host can tell "loading" from "won't load".

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

**The sandbox's side:** `sandbox.modules(place)` today returns `{ url, integrity }`; it may also return `{ code }`, a `Blob` the host has checked, which the sandbox sends as it sends compiled source, with the plain binding (spec §7.6).

**What it costs.** The text isn't copied per frame: a `Blob` posted to a frame shares its bytes with the host's page, and the frame makes its own `blob:` URL from it (a `blob:` URL is its maker's origin's only, so the host's page can't make it for the frame). What every frame pays is parsing and compiling the module, as with a URL; what O1 loses is the browser's code cache, which skips compiling a script it has seen at the same URL before. The hard cases:

| Case | What happens with O1 | With URLs (M1, O3) |
|---|---|---|
| Mermaid (2.7 MB as one file, minified) used twice in host mode | one copy of the bytes; two frames each parse and compile it | the same, with the second compile from the code cache |
| An editor rebuilding a widget's frame as it scrolls (sandbox.md Q6) | the module is compiled again each time the frame boots | from the code cache after the first |
| A visit after the first | the tarball from the resolver's cache, compiled again | the file from the HTTP cache, compiled code too |
| Many uses of a small module (ten charts, 50 KB each) | ten compiles of a small file: little | about the same |

So O1's extra cost is compiling a large module again where a URL would have it cached, about the cost of the first compile each time. A host that minds (an editor with Mermaid) uses O3, whose files are URLs; the measurements belong in the implementation (as sandbox.md Q9), with Mermaid as the case.

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
  | A version's files (the tarball, checked and unpacked: the documents and the files they import, not the rest) | its integrity | memory, for the page's life; Cache Storage, across visits and offline | until the host's cache is cleared: a version's tarball never changes | there is no Cache Storage: a page with an opaque origin (an application's sandboxed editor, sandbox.md decision 2), a private window that refuses storage, or a host that turns it off; then memory only |
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
- **Each built module is one file**, with no relative imports (R2, O1); its only imports are the runtimes the frames give (React, React DOM, Svelte). `npm create @compmd/pack` already builds one file per component; it should also turn off code splitting, so a library that loads parts of itself on demand (Mermaid does) stays in the one file. What that costs, and what chunks would need, is R7.
- **Libraries are bundled in each pack** that uses them, so ten chart packs that use d3 bring ten copies of the parts of d3 each uses: ten downloads, cached by each pack's integrity, and ten compiles in a frame that runs all ten. d3 is about 280 KB minified in full, and a chart pack's bundler keeps only the modules it imports (scales, shapes, axes: tens of KB), so ten such packs cost hundreds of KB, not megabytes. The cases where sharing pays are large libraries that several packs use (three.js, KaTeX, a chart library used whole). Sharing them needs one copy loaded once per frame and found by name: the import map's job, which today is fixed when the frames are built (its hash is in the policy), so a library from the registry can't join it at run time. That is S3, under *Not in this change*, and it meets an open spec question: whether a pack may name the libraries it expects from the host.
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

- it imports `@acme/report-kit@2`: containers written in CompMD (a 20 KB tarball), which itself imports `@acme/charts@3` (five Svelte charts, built modules with their parts of d3, 150 KB unpacked) and `@kit/mermaid@1` (one 2.7 MB module, a 0.8 MB tarball);
- the report uses five charts and two diagrams.

**Fetching**, the first visit. Pack imports form a tree, and each level waits for the one above it, since only the parent's document says what it imports; packs on one level are fetched together.

| Round | Requests | About (100 ms to npm, 20 Mbit/s) |
|---|---|---|
| 1 | `report-kit`: versions, tarball | 0.2 s |
| 2 | `charts` and `mermaid` together: versions, tarballs | 0.2 s, plus 0.4 s for Mermaid's tarball |
| Then | nothing: no further imports | |

About 0.8 s until every pack is there. The text, the tables and the CompMD containers render at once; the charts and diagrams are pending (decision 7) until their packs arrive. An import with an exact version and `integrity` skips its versions request (npm's tarball URL follows from the name and version); a host's proxy (F1) could resolve a whole tree in one request, a later addition. On a later visit, nothing is fetched but expired versions lists (R4).

**Frames.** A pack's imports don't make frames: frames follow trust (sandbox.md decision 1), so they are at most three deep (the host's page, a permissive frame, a strict frame), whatever the depth of the pack tree. Here, in host mode: seven strict frames (five charts, two diagrams) and the compile frame; in frame mode, one page frame and the compile frame.

**Memory and compiling**, estimated, to be measured when built (sandbox.md Q9):

| | Host mode | Frame mode |
|---|---|---|
| Pack files in the host's page | 3 MB of `Blob`s, shared with the frames | the same |
| Frames | eight, each a few MB for its document, the sandbox's runtime and Svelte's | two |
| Mermaid | compiled twice (once per diagram's frame), tens of ms each on a laptop, several times that on a phone, and tens of MB each once it draws | compiled once |
| Charts | five small compiles | in the page's frame |

So what grows with a document is frames, which the host chooses (sandbox.md Q1), not the depth of its packs; a large module costs a compile per frame that uses it, which frame mode or K3's code cache (R7) keeps down.

## R9. What changes

**The spec:** §6.4 *The processor gets packs*: the exception for a host that lets the reader's browser fetch from a registry its configuration names (R1); §15 *Privacy*: the same, with what it discloses; §6.4 *Built modules*: a built module SHOULD be one file, and MUST be for a host that loads it as text; §11 and §12.3: pending uses (decision 7), a stage 4a that may finish after a first render, and the expanded tree's `pending` mark; §4.5: a pending use's fallback, with `aria-busy`. F026's row keeps its cases. Data that arrives late could be pending the same way; that is not in this change.

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
- Registries other than npm's protocol. Nearly every registry a host would use speaks it: Verdaccio, GitHub Packages, GitLab, Artifactory, Nexus, AWS CodeArtifact (those that need a token only behind the host's proxy, F1, never with `direct`), and JSR through its npm endpoint (`https://npm.jsr.io/`, which answers cross-origin requests, checked 2026-10-31), where `@std/path` is named `@jsr/std__path`. A registry that doesn't (a CDN such as esm.sh) is a file server, not a registry: F3.
- Pack chunks (R7): one file per module first.
- Searching for packs from an editor: one request (`GET <registry>/-/v1/search?text=keywords:compmd-pack+<words>&size=20`, which npm answers cross-origin, cached for 300 seconds, with each package's name, version, description and links), a check of each result's `compmd` field (R6), and a picker that writes the `md.import` entry with its range and `integrity`. Showing a pack's components before importing it means resolving it. A later change; private registries may have no search (GitHub Packages has none).
