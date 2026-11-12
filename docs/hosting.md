# Hosting CompMD

**Status: proposed (2026-11-10).** What every host does to show CompMD documents with components, split into layers, so that a host on any platform (a desktop app, a web app, a site that shows documents others wrote, a product's app framework) writes only the part its platform needs, and none of the parts that decide safety. It builds on [components from apps and packs](components-from-apps-and-packs.md) (where component code comes from, at what trust), [packs from a registry](packs-from-a-registry.md), [the configuration and `@compmd/vite`](vite.md) and [running components in the sandbox](running-components-in-the-sandbox.md), and is the parent of [the Tauri plugin](tauri-plugin.md) and of `@compmd/vite`'s output.

## Why

Two hosts built so far (a Vite web app and a Tauri app) did the same work twice, a little differently each time: where the frames come from and at which version, how they are served and with which headers, how the page learns where they are, where a pack's code is stored and how it reaches a frame, and how the host checks it all in a real browser. Each of these matters for safety or for speed, and each was found the hard way once (WebKit's `'self'` on a custom scheme, its cache of custom-scheme responses, a second copy of Shiki in an app's bundle). The next host (an Electron app, a site that renders documents people push, a wiki that lets pages embed CompMD) would find them again. This design names the work once, and says which of it is shared code and which is the platform's.

## The work

| # | Layer | What it does | Platform? |
|---|---|---|---|
| 1 | **Configuration** | which packs documents get, at what trust, from which registries | no |
| 2 | **The bundle** | everything built before anyone reads: the frames, the runtimes and libraries, the packs the host installs, and a manifest | no |
| 3 | **The pack store** | every pack a document may import, from wherever it comes, checked, kept once, by its place | no, but its storage is |
| 4 | **Serving** | the bundle and the store from an origin of their own, under one contract | **yes**: the adapter |
| 5 | **The page's configuration** | where the frames are, which packs the page has, how they load | **yes**, the origin only |
| 6 | **The check** | the host proves it all in its real browser or webview | no, but it runs there |

## Decisions

1. **Four layers are shared code; a host writes an adapter for the other two.** The shared layers ship as CompMD's packages, in JavaScript and, where a host processes documents in Rust, in Rust:

   | Layer | JavaScript | Rust |
   |---|---|---|
   | configuration | `@compmd/vite/config` (`readConfig`, `policyOf`, the schema; [vite](vite.md), V5) | `compmd-host` reads the manifest, which carries the policy (decision 3) |
   | bundle | `@compmd/vite` (the plugin), and `buildBundle` for a host without Vite | `compmd-host`: the manifest and the bundle's files |
   | store | `@compmd/registry` (`Registry`, `resolvePacks`, `PackCache`, the proxy): its I/O, over the rules `compmd-host` compiles to WebAssembly (H1) | `compmd-host`: the rules (resolving, the policy, checking a tarball, reading it), and `compmd::host::Packs` over the bundle and the cache |
   | serving contract | `respond(request)` over the Fetch API, in `@compmd/host` | `respond(request)` over `http` types, in `compmd-host` |

   An adapter is the platform's few lines: where the origin is, how a request reaches `respond`, how the page gets the origin, and where the store keeps what it fetches. `tauri-plugin-compmd` is the Tauri adapter; an Electron or a web adapter is the same size. A host never writes a policy, a header, a path rule or an import map.

2. **One configuration, `compmd.config.json`, for every host** ([vite](vite.md), decision 1): the packs that get more than the strict sandbox or are ambient, `imports`, `deny`, `maxPackSize`, `libraries`, and `registries` by scope with `fetch`. A desktop app reads the same file as a web app; nothing about packs is configured in an adapter.
   - **A desktop app built with Vite** (Tauri's and Electron's templates are) uses `@compmd/vite` as a web app does, with two options a web app doesn't need: `out`, the folder the adapter embeds or serves instead of the web output, and `source`, the custom scheme's form (decision 3). With `out`, the dev server serves no bundle of its own, since the adapter's scheme does. The configuration sits beside the page's `package.json`; Rust reads what it decided from the manifest (decision 3), never the file.
   - **A host built without Vite** calls `buildBundle({ config, out, source })` from its build, with the same file.

3. **The bundle is built once, before anyone reads, from the packages the host's page imports.** The page imports `@compmd/sandbox` from npm, so the frames are built from that same package: the page and its frames speak one frame protocol by construction, one version is pinned (in the app's `package.json`), and nothing ships twice. `@compmd/vite` builds it, and `buildBundle({ config, out })` for a host without Vite; both write:

   ```
   compmd/
     manifest.json
     frames/<hash>/              strict.html, permissive.html, runtime.js, highlight.js, math.js, chunks/, libs/
     packs/<name>@<version>/     each installed and folder pack's files, as published
   ```

   - **The frames' policies name where they are served** when the origin is a custom scheme, whose `'self'` WebKit reads as nothing in a sandboxed frame: the bundle is built with that scheme's form as `source` (`compmd({ source: "compmd://localhost/" })`), which the manifest records, so an adapter serves the scheme it was built for. On http and https origins no source is needed.
   - **The frames under a content hash** (the first 16 hex digits of a SHA-256 over each file's path and bytes): a new build is a new path, so no cache (WebKit's of custom schemes, a CDN's, a browser's) serves an old frame, and every response is cached for good. The runtimes, Shiki and Temml are in there once, for the frames and for a host-mode page ([libraries in the sandbox](libraries-in-the-sandbox.md)), with the libraries the configuration offers (`buildLibraries`).
   - **Packs by their place** (spec §6.4: the pack, its version and the file's path), not under the build's hash: a published version never changes, so a pack keeps its URL across builds and across sources (decision 5).
   - **Compiled at build time**: the source components (`.jsx`, `.tsx`, `.svelte`, `.ts`) of every pack in the bundle, installed or a folder of the app, ambient or not, are compiled once, here, into built modules beside them, so no reader compiles them: `<path>.js`, with its `sha384` in the manifest's `modules`, compiled by the frames' own compiler (`@compmd/sandbox/build`'s `compile`), so the output is what a frame would compile. A Svelte component's styles are in the same module, its `css` export, which the frame adopts: one file and one hash, since a strict frame fetches nothing it could load a stylesheet with. The processor names a pack's source component by its `place` too, and the host runtime asks the host for that place's module before it compiles the source, which a frame then loads checked and mounts through the component's binding; a component that imports library packs is still compiled from its code, since a frame writes those imports into code it holds (H5); a trusted folder pack is compiled by the app's own bundler instead, since it runs in the app's page ([vite](vite.md), decision 4). Packs that aren't in the bundle come built already or are compiled once and kept (decisions 4 and 6): a registry pack ships built modules ([packs from a registry](packs-from-a-registry.md), decision 6), a host's published packs are built when the host publishes them, and a registry pack that ships source anyway, like a live pack, is compiled in the compile frame once per source.
   - **Built output, never committed:** the bundle is written where the app's build writes (the web output, or the folder an adapter embeds), which the app's `.gitignore` lists. Each app ships its own copy, as it ships any dependency in its build: a desktop app needs it offline, and browsers partition their caches by site, so a copy shared between sites on one CDN wouldn't save a reader a download.
   - **`manifest.json`** is the bundle's one description, which every layer and both languages read: `format`; the frames' `hash`; each pack's `name`, `version`, registry `integrity` (from the app's lock file, [vite](vite.md) V1), its built modules with their `sha384`, its trust and whether it is ambient; the libraries with their versions; and the processor's `policy` and `ambient` (`policyOf`). `virtual:compmd` is made from it.

4. **A document's components come from five sources; the pack store holds the three that are packs.**

   | Source | Example | Where it is | How a frame gets its code |
   |---|---|---|---|
   | **the document's own code** | a `component=` fence, `./chart.jsx` (spec §6.1, §7.1) | the document, its files | as code, compiled in the compile frame (decision 6) |
   | **the host's live packs** | a host's own pack that its users edit (unpublished, changing) | the host's own storage | as code, like the document's, since it changes; never by a place |
   | **bundled packs** | `@comp-markdown/mermaid` installed with the app, the app's folder packs | the bundle | by URL, from `packs/<place>` |
   | **registry packs** | any pack a document imports from npm, a private registry, a mirror | fetched when a document needs it, checked, kept in the store | by URL, from `packs/<place>` (decision 5) |
   | **the host's published packs** | a host's marketplace whose published versions its service already keeps | the host's own store of published versions | by URL, from `packs/<place>` |

   - **A host's published packs are a registry.** A host whose service keeps published pack versions (frozen, with their integrity), and may already bring them to the reader's device, implements `Registry` over that store: one more registry, by scope, in the configuration's `registries`. The store reads its files there, so a version is kept once, where the host keeps it, and a pack already on the device needs no network. A host that runs an npm-compatible registry for its marketplace names its URL instead, and the store fetches and caches it like any other.
   - **Order is the configuration's**: bundled and folder packs first, then `registries` by scope, as `.npmrc` (spec §6.4); a document never names a registry.
   - **Trust doesn't depend on the source.** A pack the app installed, or the host's marketplace serves, is a registry pack all the same: strict unless the configuration gives it more, never trusted, which only a folder pack built with the app may be ([vite](vite.md), decision 4; spec §15).

5. **One address per pack file, whatever its source: `packs/<name>@<version>/<path>` on the host's origin.** `respond` serves it from the bundle when the bundle has that version, and from the store's cache otherwise; the store's `PackCache` keeps a release's files by its integrity and, when it keeps one, the place it belongs to, so `respond` can find it. A built module then loads by URL with its `sha384` (sandbox, decision 7), cached for good like a frame, for every document and every reader of the host:
   - this replaces giving a registry pack's module as code (`{ code: Blob }`, [packs from a registry](packs-from-a-registry.md) R2) wherever the host serves its store, which every adapter does; `{ code }` stays for a host with no origin to serve from;
   - **except a module that imports a library pack**, which a frame must write before it runs ([sharing library packs](sharing-library-packs.md), decision 8): it is given as code, from the same stored file.

6. **Code is compiled once, and the result kept by what was compiled.** Source a reader's frames must compile (the document's own, a live pack's) is compiled in the compile frame, as today, and kept in a `CompiledCache` by a key: SHA-256 over the language, the runtime, the compiler's version and the source. The same component, in another document or another view, isn't compiled again. Compiled output is still code from a sandbox, never trusted for being cached: it runs where its source would. The cache's storage is the adapter's (decision 1): Cache Storage on the web, the app's data folder on the desktop. A pack's source is compiled at build time instead (decision 3).

7. **Processing is where the host wants it, over the same store.** A host processes in the page (`@compmd/processor`, with `resolvePacks`) or in Rust (`compmd`, with `compmd-host`'s `Packs` over the manifest, the bundle and the cache); both read the same manifest and the same stored files, so what the processor checks is what the frames run. A host that processes in Rust and fetches registry packs needs a registry client in Rust (H1).

8. **The serving contract, once, as a function.** `respond(request)` answers:
   - only `GET`; any other method is `405`, with why in plain text;
   - only `frames/<hash>/<path>` of the bundle's hash, and `packs/<name>@<version>/<path>` of a pack the bundle or the store holds, with no empty, `.` or `..` part; anything else, the manifest included, is `404`, with why; every refusal with `Access-Control-Allow-Origin: *`, so a page reads why;
   - each file with `Content-Security-Policy: sandbox allow-scripts`, `Access-Control-Allow-Origin: *` (frames are opaque origins), `Access-Control-Expose-Headers: *` (a host's check reads them), `X-Content-Type-Options: nosniff`, its type by extension, and `Cache-Control: max-age=31536000, immutable`.

   **The origin is apart from the page's** wherever the platform gives one: a custom scheme in a desktop app, a user-content domain on a site that shows others' documents. A static web host with no second origin serves the bundle from its own (`compmd/` under the app, as `@compmd/vite` does), since the frames' `sandbox` attribute gives them an opaque origin there too. The registry proxy for `fetch: "host"` (`registryProxy`) is mounted on the same origin, at `registry/`.

9. **The page's configuration is the bundle's, plus the origin from the adapter.** What the bundle knows is fixed at build time and reaches the page in its own bundle (`virtual:compmd`: the request's packs and policy, the sandbox's paths and modules, all relative to the origin). The origin is the one thing only the running host knows (a scheme's form differs by platform, a site's user-content domain by deployment), so the adapter gives it: `globalThis.__COMPMD__ = { origin }`, set by a Tauri initialization script, an Electron preload, or a `<meta name="compmd-origin">` a server writes. `createSandbox` reads it; the page passes nothing.

10. **The page's content security policy comes from the bundle too.** `cspFor(origin)` gives the sources a host's page adds: `frame-src` the origin, `script-src` and `connect-src` `<origin>/frames/` (a host-mode page loads Shiki and Temml from there), `connect-src` `<origin>/registry/` with `fetch: "host"`, and `frame-ancestors 'none'`. Never `<origin>/packs/` and never the whole origin: a pack's code runs in frames only. A host whose policy is compiled from a file (`tauri.conf.json`) pastes it; a server sets it from the function.

11. **Every host runs one check, in its real browser.** `@compmd/sandbox/check` is a page and a document that exercise the host: a strict frame loads from the origin; in it the origin is opaque, there is no host API (the adapter names what to look for: `__TAURI_INTERNALS__`, Electron's `ipcRenderer`), and `fetch` is refused; WebKit's `ipc` message handler is absent, or, where WebKit gives it to every frame and the host refuses a frame's messages (Tauri, by its invoke key), the adapter's check posts a call through it that must run nothing ([the Tauri plugin](tauri-plugin.md#appendix-webkits-ipc-handler-in-a-frame)); a code block is highlighted; a bundled pack's module and a stored one load by URL and draw; files arrive with exactly the contract's headers; a path outside the contract is `404`. It reports passed or failed with what failed, for the adapter to turn into an exit code (the Tauri example's `--check`) or a test. An adapter's CI runs it in the webview or browser that matters (WebKit first, which has the hard cases).

12. **The spec stays as it is, but for one clarification.** It already says what a sandbox guarantees and not how it is built (§15), and lets a processor load a built module by its place, checked, and cache it by that place (§6.4). It gains, in §6.4: a pack an application installs, or a host serves from its own store, is a registry pack all the same: it keeps its integrity and its trust is the configuration's; caching by place holds however the processor got the pack.

## Hosts

| Host | Processes | Origin | Adapter | Registry packs | Store's storage |
|---|---|---|---|---|---|
| **A Tauri app** | in Rust or the page | a custom scheme (`<scheme>://localhost`, `http://<scheme>.localhost` on Windows and Android) | `tauri-plugin-compmd`: the bundle compiled in, `respond` on the scheme, the origin in an initialization script | fetched by the app | the app's data folder |
| **An Electron app** | in the main process or the page | a privileged scheme (`protocol.handle`) | `@compmd/electron` (later): `respond` on the scheme, the origin in a preload | fetched by the main process | the app's data folder |
| **A web app built with Vite** | in the page | its own origin, `compmd/` | `@compmd/vite`: the bundle in the output, the dev server's `respond` | through its proxy (`fetch: "host"`) or the browser (`direct`) | Cache Storage |
| **A site that shows documents others wrote** (a code host's README, a wiki page) | on its servers, in Rust or Node | a user-content domain | `respond` in a worker or the server, the origin in a `<meta>` | by its servers, cached for every reader (spec §15, privacy) | its object storage or CDN |
| **A product's app framework** (a wiki's or an issue tracker's apps) | in the app's page or its backend | the framework's static resources for apps, when they can carry the contract's headers; else its own origin | the framework's adapter over `respond` | by its backend | its storage |

## What changes

- **`@compmd/vite`:** the layout of decision 3 (`frames/<hash>/`, `packs/<place>`, `manifest.json`); packs' sources compiled at build; `buildBundle` for hosts without Vite; `cspFor`; a headers file (`_headers`) for static hosts that read one (H2); paths in `virtual:compmd` relative to the origin.
- **`@compmd/sandbox`:** `createSandbox` takes the origin from `__COMPMD__` and the rest from `virtual:compmd`; `CompiledCache`; stored registry packs by URL (decision 5); the check page.
- **`@compmd/registry`:** its rules from `compmd-host`'s WebAssembly build (H1), its API unchanged; `PackCache` keeps the place a release belongs to; a folder-backed cache for Node and desktop adapters.
- **`@compmd/host`** (new, JavaScript): `respond` over the Fetch API, with the dev server, workers and Node using it.
- **`compmd-pack`** (new crate): the names packs and their files are known by (`PackName`, `ReleaseId`, `FilePath`, `Place`, `Integrity`), each checked once, when it is made; the processor uses them in its host API (`Packs`, `Release`, `Module`, `LibraryModule`, `NeededPack`, `Ambient`, the policy) and re-exports them as `compmd::pack`, and `compmd-host` uses the same, so a pack's name or a file's place means one thing in every crate. The processor's JSON is unchanged.
- **The processor:** a pack's source component's `HostComponent` names its place too, beside its `code`, so a frame loads its built module when the host has one (decision 3), and compiles the code otherwise.
- **`compmd-host`** (new crate): the manifest, the bundle's files, the cache, `respond`, `compmd::host::Packs` over them, and the registry rules (H1), with its WebAssembly build.
- **`tauri-plugin-compmd`:** the Tauri adapter ([its design](tauri-plugin.md)).
- **The spec:** decision 12's clarification of §6.4.
- **The guides:** a chapter on hosting, from the hosts above, with each adapter's page.

## Questions, with proposed answers

**H1. A registry client in Rust.** A host that processes in Rust (decision 7) must get registry packs there, since the processor's `Packs` is synchronous and reads what the host already holds.
- *What it is:* the resolver's loop (ask the processor's `sources` for what's missing, resolve each range, fetch, add, until nothing is), `NpmRegistry` (a pack's metadata with the short `Accept`, its tarball), the policy before any download (deny, `imports: listed`, the size cap against the metadata's unpacked size), the tarball checked against `dist.integrity` (sha512) and against an import's pinned `integrity`, and the tar reader: [resolving registry packs](resolving-registry-packs.md), decisions 2 to 7 and 11, which `@compmd/registry` builds today.
- *Options:* (a) port it to `compmd-host` and keep `@compmd/registry`'s JavaScript too: two implementations of the same rules, kept equal by shared fixtures; (b) run `@compmd/registry` in the host's page and hand the checked files to Rust through the adapter, so one implementation exists, but processing waits on the page, and a host with no page (a server, a CLI) can't fetch; (c) **one implementation, in Rust:** the rules (the resolver's loop, ranges as the processor matches them, the policy before any download, the integrity checks, the tar reader) move into `compmd-host`, which `@compmd/registry` uses as WebAssembly, as `@compmd/processor` uses the processor; `@compmd/registry` keeps what is JavaScript's own (`fetch`, Cache Storage, the proxy as a fetch handler, its API), and `compmd-host` takes an HTTP client and a cache from its host (traits, so the crate brings neither).
- *Proposed:* (c). Nothing is written twice: `@compmd/registry`'s API stays, its rules are the crate's, and its tests run against the WebAssembly build. The cost is a WebAssembly build of `compmd-host` beside the processor's, and the tar reader and checks moving from TypeScript to Rust once.
- *Until then:* a Rust host offers bundled packs and its own `Registry` implementations only (decision 4); an import of any other pack is F026, *not offered by this host*.

**H2. A web app on one origin.** A static host (a site on a static file host, the playground) has no second origin to serve the bundle from (decision 8).
- *What it costs:* a frame document opened on its own, as a top-level page, has the app's origin, since `sandbox` applies only inside the frame's `iframe`, and the frame document's own `Content-Security-Policy: sandbox allow-scripts` header (decision 8) is what still makes it opaque then. A static host that can't set headers loses that too.
- *Options:* (a) allowed, with the header where the host can set it (`_headers` and the like, which the bundle's build writes, decision 3), and the check (decision 11) reporting a host that doesn't send it; (b) refused: a host must have a second origin.
- *Proposed:* (a), since it is what `@compmd/vite` hosts do today, safely inside frames; the check says when the header is missing, and the hosting guide recommends a second origin wherever there can be one.

**H3. A live pack's code across views.** A host's live pack (one its users edit, unpublished) changes, so it has no place to be cached by.
- *Options:* (a) as code, its source compiled once per content and kept by its source's hash (decision 6), so an unchanged live pack isn't compiled again in another view; (b) a pseudo-version per content (`0.0.0-live.<hash>`), served at a place like a published one, which caches the same but makes up versions a registry never published, which spec §6.4's integrity rules don't expect.
- *Proposed:* (a). It gets a place only when the host publishes a version of it.

**H4. The bundle's layout changes for `@compmd/vite` hosts.** Today a Vite host's output has `compmd/strict.html` at a fixed path; decision 3 moves the frames under `frames/<hash>/`.
- *What breaks:* nothing a host wrote, if it uses `virtual:compmd` (its `sandbox` gives the paths) and `cspFor` (decision 10); a host that wrote `compmd/` paths into its own CSP or server rules updates them.
- *Proposed:* 0.7.0's release notes say so, with the CSP change; there is no compatibility layout, since the old one is what lets a cache serve a stale frame.

**H5. Built modules that import a library pack.** A frame writes a library pack's import as that library's URL in the module's code before it runs it ([sharing library packs](sharing-library-packs.md), decisions 6 and 8), which it can't do to a module it loads by URL.
- *Options:* (a) such a module reaches its frame as code, from the same stored file, as decision 5 says; (b) the bundle's build writes the import as the library's place for bundled packs, which works only when the build knows the version the processor will choose per document, which it doesn't.
- *Proposed:* (a). The cost is that such a module isn't cached by the browser across views; the store still keeps it once, and the frame's `blob:` URL is made from it each time.

**H6. Where this design's contract is written.** The serving contract, the bundle's layout and the page's configuration are what CompMD's own packages agree on, not what any processor must do: the spec leaves how a sandbox is built to implementations (§15), and a host that doesn't use `@compmd/sandbox` has no frames to serve.
- *Proposed:* a design here, not a companion of the spec. If hosts appear that build their own frames and want to share packs' stores with CompMD's, the contract can move into a companion then, as the registry's did once a second host needed it.

## Appendix: cases that challenge each decision

| Decision | Case | What happens |
|---|---|---|
| 1 | An Electron app shows documents | It writes `@compmd/electron`'s few lines: `protocol.handle` on a privileged scheme passing requests to `respond`, a preload setting `__COMPMD__`, a store folder; everything else is shared. |
| 1 | A host has its own frames and compiler | It uses the protocol, the frame runtime and the host runtime ([running components](running-components-in-the-sandbox.md), decision 6) and none of the bundle; this design doesn't apply to it. |
| 2 | A desktop app wants a pack permissive, with the network to its tiles | `compmd.config.json` says so, as a web app's does; the adapter has no setting for it. |
| 2 | A Tauri app's page is built with Vite, and its Rust with Cargo | `@compmd/vite` with `out: "src-tauri/compmd"` writes the bundle; Cargo embeds it; neither reads the other's configuration. |
| 2 | A host's configuration lists a pack the app doesn't install and names no registry | The build stops: the configuration names what can't be had ([vite](vite.md), decision 7). |
| 3 | The page's `@compmd/sandbox` is upgraded and the frames aren't rebuilt | Impossible with the bundle: it is built from that package in the same build. |
| 3 | Two builds with the same frames | The same hash, so the same paths: caches keep what they have. |
| 3 | A pack in the bundle has a `.svelte` component | Compiled at build time into a built module beside it; no reader compiles it. |
| 3 | A developer runs `git status` after a build | The bundle's folder isn't listed: it is build output, in the app's `.gitignore`. |
| 3 | A Tauri app on Windows and macOS | One bundle: the `source` names the custom scheme, which on Windows (served over http) matches nothing and is harmless. |
| 4 | A document imports `@acme/charts@^1`, installed at 1.4.0 with 1.5.0 on the registry | Bundled first (configuration order): 1.4.0, which matches; the registry isn't asked. |
| 4 | A document pins `@acme/charts@1.5.0` with its `integrity` | Not bundled at that version: fetched from the registry by scope, checked against the pin, kept in the store. |
| 4 | A host's marketplace pack already on the device | Its `Registry` reads the host's store; nothing is fetched or copied. |
| 4 | An installed pack configured `trusted` | Refused at build: only a folder pack of the app may be trusted ([vite](vite.md), decision 4). |
| 4 | A host's live pack that a user edits | Given as code; each edit is a new source, compiled once (decision 6). |
| 5 | The same pack version, bundled and also fetched | One address: `respond` serves the bundle's copy; the store keeps none. |
| 5 | A stored file whose bytes no longer match | The frame checks the `sha384` before it runs anything, and fails with the sandbox's error; the store drops the release. |
| 5 | A pack module imports the library pack `@kit/three` | Given as code from the stored file, so the frame can write the import (H5). |
| 6 | The same JSX fence in two documents | One compile: the second finds it by its source's hash. |
| 6 | The compiler is upgraded | Its version is in the key, so nothing compiled by the old one is used. |
| 6 | Cached compiled code is tampered with on disk | It runs where its source would, in its frame, at its trust: nothing more than the source could do. |
| 7 | A host processes in the page, another in Rust, on one bundle | Both read `manifest.json` and the same stored files: what the processor checks is what the frames run. |
| 8 | A `POST` to the origin | `405`, with why. |
| 8 | `GET /manifest.json` | `404`: the manifest is the host's, not the frames'. |
| 8 | `GET /packs/@acme/charts@1.4.0/../../secret` | `404`: no `..` part. |
| 8 | A pack not held at all | `404`, with why; the frame reports that its module didn't load. |
| 9 | A site deploys the same build at two user-content domains | The build is the same; each server writes its own `<meta name="compmd-origin">`. |
| 9 | A Tauri app on Android | The initialization script gives `http://<scheme>.localhost`; the page's bundle is unchanged. |
| 10 | A host adds `<origin>/` to its page's `script-src` "to be safe" | The check fails it: a pack's code could then run in the host's page. |
| 11 | WebKit starts sending a header the contract doesn't name | The check, which compares headers exactly, fails in CI before a release. |
| 11 | An adapter forgets to keep the invoke key (or a preload API) out of frames | The check's frame looks for the host API it names and fails. |
| 11 | WebKit gives a strict frame Tauri's `ipc` message handler | Expected: the check leaves it to the host (`webkitIpc: "guarded"`), and the Tauri example's probe posts a call through it, which must run nothing. |
| 12 | A document pins the integrity of a pack the host installed | The same integrity as the registry's (from the lock file), so the pin holds; installing doesn't make a different pack. |

## Alternatives

- **Each platform's package does all of it** (0.6.0's Tauri plugin, which brought its own frames; `@compmd/vite`, which wrote its own layout). Each was right for its host, and they differed: one served fixed paths, the other hashed ones; one took the frames from npm, the other from the crate.
- **The frames from the crate or package of each adapter.** No build step on the host, but a second place for the frames' version, checked only when a page loads them, and a host with libraries or packs builds a bundle anyway.
- **Registry packs always as code** (R2 as it is). No serving, but every reader's page holds each module in memory, uncached across views, and a desktop app keeps its packs only for a session.
- **The spec defines hosting.** It would say how a browser's sandbox is built, which §15 leaves to implementations on purpose: a server's sandbox, or a native one, looks nothing like a frame.
