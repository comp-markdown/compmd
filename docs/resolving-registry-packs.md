# Resolving registry packs

**Status: accepted (2026-11-01).** How `@compmd/registry` resolves, fetches and checks packs. It builds decisions 1 to 3 and 5 of [packs from a registry](packs-from-a-registry.md) (R1 to R5), as the companion [*CompMD packs on a registry*](../compmd-registry-v1.md) states them (sections 3 and 4), which decided *what* a host does; this design decides *how*: a host gives `resolvePacks` its request and its registries, and gets back the packs the document needs, fetched and checked. Library packs (R9) are [their own task](packs-from-a-registry.md#r9-libraries-packs-share-s3), and so is the sandbox's side, built modules given as code (R2).

## Proposed decisions

1. **`resolvePacks` gives a new request, not parts of one.** `const { request, modules, problems } = await resolvePacks(request, options)`: the same request with the fetched releases added to `packs` and, for a pack the size cap refuses, `policy.refused` added to, so the processor says why (F026). The host passes `request` to `process` or `OpenDocument` as it is. `modules(place)` gives a built module's checked text as `{ code: Blob }`, for the sandbox once it takes code ([its task](packs-from-a-registry.md#r2-how-a-registry-packs-module-reaches-its-frame)); `problems` lists what couldn't be fetched (a registry that didn't answer, a version that 404s), for the host's logs. What the document sees is always the processor's diagnostics.
2. **The loop is the processor's `sources`, as the design says.** Each round asks `sources(request).packs`, resolves every pack listed at once (one round per level of the pack tree, R8), adds what it got, and asks again. It ends when nothing is listed, or nothing new was got: a pack the registry doesn't have stays listed, and its import stays F026 (*no release matches*). `@compmd/processor` is a peer dependency; the host loads its module first, as it does for `process`.
3. **Ranges match as the processor matches them.** The processor's range rules (Rust's `semver`, with `1.4.2` exact, spec §6.1) and npm's `semver` package differ at the edges, and a resolver that picked a version the processor then rejects would fetch for nothing. So `@compmd/processor` gains `newestMatching(range, versions)`, the processor's own choice, and the resolver uses it. Nothing new is bundled for semver.
4. **The policy, before any download** (companion, section 3, step 3):
   - `imports: listed`, `refused` and `@compmd/`: the processor doesn't list those packs, so they are never asked for.
   - **`deny`**: the version the range resolves to is denied, so it isn't downloaded. The resolver adds it as a release with no files, and the processor, which checks a denial before it reads a pack, gives F026 with the host's reason.
   - **The size cap**, `maxPackSize` in the options, 10 MB when the host sets none ([the Mermaid pack](mermaid-pack.md), decision 7): the version's `unpackedSize` from the registry's metadata. Over the cap, nothing is downloaded and `policy.refused` gets the pack, with its size and the cap in the reason. Metadata without `unpackedSize` counts as over the cap: what can't be measured isn't fetched.
5. **A tarball that fails its check is F035, through the processor.** The tarball is checked against `dist.integrity` with `crypto.subtle` (sha512, the only algorithm npm publishes for current packages; metadata with no `sha512-` integrity isn't fetched). A pinned `integrity` that differs from `dist.integrity` already gives F035 in the processor (the release is added with no files, as for `deny`). A tarball whose bytes don't match `dist.integrity` needs the same diagnostic for an import that pins nothing, which the processor can't give today. **Proposed:** a release in the request may carry `broken: "<why>"`, and an import that resolves to it is F035 with that message, its names dropped. This is a small change to the processor and its request type; the spec's F035 row already covers it ("the files it is loaded from are checked against the pack's integrity").
6. **A release keeps its text files.** The tarball is read whole (it is what the hash covers): every file under `package/` that is valid UTF-8 is kept, by its path without `package/`; any other (an image, a font) is dropped, since the processor reads text and a built module is text. `package.json` is the tarball's, never the metadata's (companion, section 3, step 5). Keeping all text files, rather than working out which ones the documents import, keeps the reader simple; `files` in a pack's `package.json` keeps them few (companion, section 2). Path traversal (`..`, an absolute path), links and entries outside `package/` are dropped.
7. **`Registry` is two methods, `NpmRegistry` the one implementation.**
   - `versions(name)`: each version with its tarball URL, `integrity` and `unpackedSize`, from npm's short metadata (`GET <url>/@scope%2fname`, `Accept: application/vnd.npm.install-v1+json`); a 404 is no versions.
   - `tarball(version)`: the tarball's bytes, from the URL the metadata gave. Checking and unpacking are the resolver's, so a host's own `Registry` can't skip them.
   - `new NpmRegistry({ url, fetch? })`: any registry that speaks the protocol; a custom `fetch` for a token or a test.
   - Registries by scope in the options, as `.npmrc` does: `{ "@kit": registry, "*": registry }`. No match and no `*`: the pack isn't fetched.
8. **`fetch: "host" | "direct"`, the host's word, enforced** (companion, section 4). Every request is `credentials: "omit"` and carries only the pack's name and version.
   - **`host`, the default:** in a browser, a registry must be on the page's own origin (the host's proxy or cache); a cross-origin URL is an error when `resolvePacks` is called (a registry is made before it knows the host's `fetch`), naming `fetch: "direct"`. A tarball URL on another origin is refused the same way, since a registry's metadata names it. On a server there is no reader to protect, and any URL is fine.
   - **`direct`:** any `https:` registry. The host tells its readers (the companion's SHOULD); the README says so.
9. **The proxy is a fetch handler.** `registryProxy({ registries })` returns `(request: Request) => Promise<Response>`, which a worker exports and Node serves through a small adapter (`toNodeListener`, for `http.createServer` and Vite's dev server, which `@compmd/vite` uses). It answers only `GET` of a pack's metadata and of its tarballs: metadata is fetched from the scope's registry with the short `Accept`, each `dist.tarball` is rewritten to the proxy's own URL, and tarballs are streamed through. Incoming cookies and credentials are never passed on; a registry's token is the proxy's own configuration (`headers`). Anything else is 404. It caches nothing itself: a host puts its own HTTP cache or CDN in front.
10. **The cache, as R4 says, behind an interface.** `PackCache` has `files(integrity)` / `keep(integrity, files)` and `versions(key)` / `keepVersions(key, list, maxAge)`. `defaultCache()` is memory for the page's life plus Cache Storage (`caches.open("compmd-registry-v1")`, entries keyed by a synthetic `https://compmd.invalid/…` URL) across visits; where `caches` is missing or throws (an opaque origin, a private window), memory only. A versions list is kept for the registry's `max-age` (npm's is 300 seconds), and a stale one is used when the registry can't be reached. An import with an exact version and an `integrity` whose files are cached fetches nothing.
11. **The tarball reader is ours, small.** `DecompressionStream("gzip")` (browsers, Node 18 and later, workers), then a tar reader of about a hundred lines: ustar headers, PAX `path` for long names, regular files only. No dependency: it reads only what a host trusts after the hash check, and is tested against tarballs `pnpm pack` wrote (the tool the release workflow packs the packages with; npm doesn't run inside this repository, whose `devEngines` names pnpm).

## The API

```ts
import { init, process } from "@compmd/processor";
import { defaultCache, NpmRegistry, resolvePacks } from "@compmd/registry";

await init();
const registries = { "*": new NpmRegistry({ url: "/compmd/registry/" }) }; // the host's proxy
const { request: ready, modules, problems } = await resolvePacks(request, { registries, cache: defaultCache() });
const processed = process(ready);
```

```ts
// A static site, said in its About page:
const registries = { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) };
await resolvePacks(request, { registries, fetch: "direct" });
```

```ts
// The proxy, in a worker:
import { NpmRegistry, registryProxy } from "@compmd/registry";
export default { fetch: registryProxy({ registries: { "*": new NpmRegistry({ url: "https://registry.npmjs.org/" }) } }) };
```

An editor calls `resolvePacks` in the background when `OpenDocument.sources().packs` isn't empty, and opens the document again with the request it gives (decision 7 of the design).

## Tests

- **Unit, in Node:** the tar reader on tarballs written by `pnpm pack` (checked in as fixtures, a few KB), long names, links and traversal dropped; the integrity check; the policy (deny, the cap, missing `unpackedSize`); the cache's memory side; `fetch: "host"` refusing a cross-origin URL.
- **Against a local registry:** a Node server in the tests that speaks npm's protocol for a few packs (a pack importing a pack, a denied version, one over the cap, a corrupted tarball, a pinned import), serving the same fixtures; `resolvePacks` end to end through `NpmRegistry`, and through `registryProxy` in front of it; the processor's diagnostics for each case (F026, F035).
- **In Chrome and WebKit** (Vitest's browser mode, as `@compmd/sandbox`): `DecompressionStream`, `crypto.subtle` and Cache Storage, against the local registry, with a second resolve served from the cache.

## Not in this change

- Library packs and the sandbox's `{ code }` (their own tasks); `modules` is given here so the sandbox's task only consumes it.
- Checking npm's signatures; registries that need a token from the reader's browser (only behind the proxy, decision 9).
- A proxy that resolves a whole tree in one request (R8, a later addition).
