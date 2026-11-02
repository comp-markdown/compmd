# Sharing library packs

**Status: proposed (2026-11-02).** How the processor, `@compmd/registry` and `@compmd/sandbox` let packs share a library pack: L2 of [packs from a registry](packs-from-a-registry.md) (decision 8, R9), as the companion [*CompMD packs on a registry*](../compmd-registry-v1.md) states it (section 5). A pack names `"@kit/three": "^0.170"` in `peerDependencies` and imports `@kit/three` in its components' code; the host fetches the library pack, chooses one version for the document, and the frame loads it once, for every pack there that imports it.

## Proposed decisions

1. **The processor chooses each library's version; the resolver only fetches.** The companion's rules (one version per document, a second refused for a `single` library, W121 on the import) end in the processor's diagnostics, and a resolver that chose would have to tell the processor what it chose and why. So the processor does what it does for packs:
   - `sources(request).packs` also lists the libraries the packs it holds name in `peerDependencies`: a scoped name that isn't a runtime and isn't a library the host offers (`policy.libraries`, spec §6.3), whose range no release in the request matches. Each is listed with `library: true` and no `integrity` (a peer range pins none).
   - A library pack's release goes in `request.packs`, as a pack's does; `resolvePacks` fetches it as it fetches any pack, under the same policy before any download (companion, section 3).
   - The processor then chooses, per document, from the releases it holds: the newest that matches every range the document's packs ask for; when none does, each range gets the newest that matches it; for a `single` library, the version the first pack to ask gets (in the order the processor resolves imports: the document's in order, each pack's own before the next), and each pack whose range it doesn't match gets W121 on its import: "`@b/charts@2.0.0` needs `@kit/three` `^0.171`, which can't have two copies in one sandbox, and this document has 0.170.0, for `^0.170`".
2. **A library pack is checked when it is chosen, and what can't be had is W121.** On the import of the pack that names it, with why: a release whose `compmd` field isn't `{ "library": path }` ("isn't a library pack"), a library the host denies, refuses or doesn't list under `imports: listed` (its reason, as F026's for a pack), one no release matches ("this host couldn't get"), a module file the release doesn't have. Not F026: the document didn't import it. The pack's components that don't use it still render.
3. **A library's own libraries are its `peerDependencies`,** chosen by the same rules, so a library that imports `@kit/math` gets the document's one copy of it. Libraries import only libraries, so a cycle is a library pack's mistake: W121 on the import, naming the cycle.
4. **The processor tells the sandbox which library each import names.** `HostComponent` gains `libraries`, for a component of a pack that names any: each import name with its library's module, `{ pack, version, path, libraries? }`, and that library's own, nested the same way. A frame needs nothing else to write a component's imports.
5. **The resolver fetches the version the processor will choose.** For a library listed with several ranges (in one round or across rounds), `resolvePacks` fetches the newest version that matches every range it has seen for that library, if there is one, else the newest that matches the range listed. It keeps those ranges for the resolve. Without this, two ranges whose newest versions differ but which share an older one would get two copies where the companion says one.
6. **The frame writes a library's name in a pack's code as that library's URL, not the host's page** (corrects R9, which said the host's page). The URL is a `blob:` URL, and a `blob:` URL works only for its maker's origin (R2): the host's page can't make one for a frame. So:
   - the `code` message gains `libraries`, each with its place (`pack@version/path`), its file as a `Blob` (from `modules(place)`, as a built module's) and its own imports' places; a component that imports any has them, by name, in `libraries`;
   - the frame loads each library once, its own libraries first, as a `blob:` module it keeps for its life; then it reads the component's text, writes each import of a library's exact name (static imports, `export … from`, and `import()` with a literal) as that library's URL, and loads the result as it loads compiled text today;
   - the parser is es-module-lexer's plain JavaScript build (`es-module-lexer/js`), in the frame runtime: the strict policy has no WebAssembly. It is 28 KB minified, not the 4 KB R9 says (that is the WebAssembly build's).
7. **A library runs at the level of the pack that imports it.** A frame gets a library only with the code of a component it runs (`codeFor`), so a strict pack's library runs in its strict frame, and a permissive pack's in its permissive frame; packs of two levels that share a library get a copy each, one per level, which is what the trust rules require.
8. **Only code the frame can write uses library packs.** Compiled source and built modules given as code (`{ code }`) are written; a built module given by its URL (`{ url, integrity }`) loads as its file, which the frame can't change, so if it imports a library pack it fails with W121, and so does a library given by URL. A host that serves pack files by URL (`@compmd/vite` today) shares libraries as libraries it offers (L1).

## What the frame trusts

Nothing new: the frame believes its parent ([the protocol](sandbox-protocol.md), decision 2), and its parent sent both the pack's checked text and the library's. Writing changes only the specifiers the parser found, each to the URL of a module the same message brought, and an import of any other name resolves through the import map as before. A bug in the writing changes which code runs in that frame only; it can't reach the network, which the strict policy still forbids.

## Tests

- **The processor, with conformance cases:** one version for two packs whose ranges share one; two versions for ranges that don't; `single` with W121 naming both ranges and the version; a library that isn't a library pack, a denied one, one not to be had; a library's own library; a cycle; `sources` listing libraries; `HostComponent.libraries`.
- **`@compmd/registry`, against the local registry:** a pack with a library, two packs with ranges whose newest versions differ but share one (one fetched), and the processor's choices and diagnostics end to end.
- **`@compmd/sandbox`, in Chrome and WebKit:** a component that imports a library given as a `Blob`, in a strict frame and in host and frame modes; two packs sharing one copy (one module instance); two versions in one frame; a library's own library; a built module by URL that imports one (W121).

## What changes

- **The companion, section 5:** a library pack's own libraries are its `peerDependencies`; for a `single` library, the first pack to ask decides; a library that can't be had is W121 with why. The changelog says so.
- **[Packs from a registry](packs-from-a-registry.md), R9:** the frame writes the URL, not the host's page; the parser's size.
- **The processor:** `NeededPack.library`, the choice, W121, `HostComponent.libraries`; its TypeScript types.
- **`@compmd/registry`:** decision 5; the README.
- **`@compmd/sandbox`:** the `code` message's `libraries`, `Runnable.libraries`, the writing in the frame; the protocol's schema and [its document](sandbox-protocol.md); the README.
