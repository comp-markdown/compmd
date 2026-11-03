<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-dark.svg">
  <img src="https://raw.githubusercontent.com/comp-markdown/compmd/main/logo/compmd-light.svg" alt="CompMD" width="200">
</picture>

# @compmd/registry

Packs for CompMD documents from an npm registry. A document imports `@kit/mermaid@1` and the host has never heard of it: `resolvePacks` asks the processor which packs are missing, resolves each range as the processor does, applies the host's policy before anything is downloaded, checks each tarball against the integrity its registry publishes, and gives back the request with the packs in it. It does what [*CompMD packs on a registry*](../compmd-registry-v1.md) (sections 3 to 5) says a host does; the design is [resolving registry packs](../docs/resolving-registry-packs.md), and for library packs [sharing library packs](../docs/sharing-library-packs.md).

**Website:** [compmd.hakanserce.com](https://compmd.hakanserce.com/), with the spec and a playground to try CompMD in.

```sh
npm install @compmd/registry @compmd/processor
```

It implements the draft of the [CompMD specification](https://compmd.hakanserce.com/spec/), version 1, and shares its version with the other `@compmd/` packages ([how releases work](https://github.com/comp-markdown/compmd/blob/main/docs/releases.md)).

**Versions:**
- **0.4.0:** `timeout`, after which `resolvePacks` stops waiting for a pack and lists it in `problems`, for a host that renders with its packs pending meanwhile.
- **0.2.0:** the first version.

```ts
import { init, process } from "@compmd/processor";
import { defaultCache, NpmRegistry, resolvePacks } from "@compmd/registry";

await init(); // the processor's module, once
const registries = { "*": new NpmRegistry({ url: "/compmd/registry/" }) }; // the host's proxy
const { request: ready, modules, problems } = await resolvePacks(request, { registries, cache: defaultCache() });
const { expanded, diagnostics } = process(ready);
```

- **`resolvePacks(request, options)`** gives `request` with the fetched releases added to `packs`, and the packs the size cap refuses added to `policy.refused`, so the processor says why (F026). It loops on the processor's `sources(request).packs`, one round per level of the pack tree, until nothing is missing or nothing more can be got. What it couldn't fetch (a registry that didn't answer) is in `problems`, for the host's logs; what the document sees is always the processor's diagnostics.
- **Rendering meanwhile.** A host that renders before its packs arrive names those it is getting in the request's `pending` (`sources(request).packs`, spec §6.4): their uses show as loading, with no F026. `resolvePacks` stops waiting for a pack after `timeout` (30 s when the host sets none) and lists it in `problems`; processed again without it pending, the document says F026. The processor never gives up on a pending pack itself.
- **`modules(place)`** gives a built module's checked text as `{ code: Blob }`, by the place the processor gives it with a host component (`{ pack, version, path }`), for the sandbox.
- **Versions** are picked with the processor's own `newestMatching`, so the release added is the one the processor takes.
- **Library packs** the packs name in `peerDependencies` are listed by the processor too (`library: true`), and fetched as packs are. For each, the version fetched is the newest that every range asked for it in the resolve matches, as the processor chooses one version per document; a version two imports resolve to at once is downloaded once.
- **The policy, before any download:** a pack the policy refuses, or isn't allowed to import, is never asked for. A denied version is added with no files, and the processor gives F026 with the host's reason. A version over the size cap (`maxPackSize`, 10 MB when the host sets none), or whose metadata doesn't say its unpacked size, isn't fetched.
- **Every tarball is checked** against `dist.integrity` (SHA-512), and against the import's pinned `integrity`. One that doesn't match is added as `broken`, and the processor gives F035. `package.json` is the tarball's, never the metadata's. A release keeps the tarball's text files under `package/`; links, binary files and paths that would leave the package are dropped.

## Registries

`new NpmRegistry({ url, fetch?, headers? })` speaks npm's protocol at any URL: npm's own, a mirror, a private registry, a proxy. Registries go by scope, as `.npmrc` has them: `{ "@kit": kit, "*": npm }`. A pack whose scope has none, with no `*`, isn't fetched. `Registry` is the interface, for a host's own: `versions(name)` and `tarball(version)`; the resolver checks and unpacks.

## Who fetches

`fetch` in the options is the host's word (companion, section 4). Every request goes without credentials and carries only a pack's name and version.

- **`host`, the default:** in a browser, every registry and tarball must be on the page's own origin: the host's proxy or cache, so the registry learns nothing about who reads which document. A registry on another origin is an error. On a server, any URL is fine.
- **`direct`:** the reader's browser fetches from the registries the host names, as a static site with no server must. The registry then learns the reader's address and the names of the packs a document imports, never the document. **Tell your readers**, for example on an *About* page.

```ts
const registries = { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) };
await resolvePacks(request, { registries, fetch: "direct" });
```

## The proxy

`registryProxy({ registries, base })` is a fetch handler: `GET <base><name>` gives a pack's short metadata, with each tarball's URL rewritten to the proxy's own, and `GET <base>-/tarball/<name>/<version>` streams that tarball. It answers nothing else, passes on nothing the reader sent, and caches nothing itself: put an HTTP cache or CDN in front. A private registry's token goes in its `NpmRegistry`'s `headers`, on the server.

```ts
// A worker:
export default { fetch: registryProxy({ registries: { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) }, base: "/compmd/registry/" }) };

// Node:
import { createServer } from "node:http";
import { toNodeListener } from "@compmd/registry/node";
createServer(toNodeListener(registryProxy({ registries, base: "/compmd/registry/" }))).listen(8080);
```

## The cache

`defaultCache()` keeps what was fetched in memory for the page's life and in Cache Storage across visits and offline, and only in memory where the page has no Cache Storage (an opaque origin, a private window). A version's files are kept by their integrity, which never changes; a pack's versions list for its registry's `max-age`, and a stale one is used when the registry can't be reached. An import with an exact version and an `integrity` whose files are kept fetches nothing. `memoryCache()` is memory only; `PackCache` is the interface, for a host's own (one on its server, behind its proxy).

## Commands

```sh
pnpm install         # in the repository root
pnpm build           # ESM and types into dist/
pnpm test            # unit tests in Node against a local registry, and browser tests in Chrome and WebKit
pnpm lint            # eslint and the type check
```

The browser tests drive the installed Chrome and Playwright's WebKit (`pnpm exec playwright install webkit`). The tests' tarballs are written by `pnpm pack` from the packs in `test/fixtures/packs/`: after changing one, `pnpm --dir test/fixtures/packs/<pack> pack --pack-destination test/fixtures/tarballs`.

## License

[MIT](../LICENSE-CODE). Copyright © 2026 Hakan Serce.
