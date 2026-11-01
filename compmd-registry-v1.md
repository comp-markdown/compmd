# CompMD packs on a registry, version 1 (draft)

A companion to the [CompMD specification](compmd-spec-v1.md), for pack authors and for hosts that fetch packs from a registry. The spec says what a pack is to a document (§6.4); this document says what a pack published to an npm registry must be, how a host reads one, and what a library pack is. It is versioned with the spec: version 1 of each goes together, and every change to it has an entry in the [changelog](CHANGELOG.md).

**Last changed 2026-11-01.**

## 1. Conventions

- The key words MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119.
- A **registry** is a server that speaks npm's registry protocol: npm's own, a mirror, a private registry or a proxy. A **host** is the application whose processor renders documents; its configuration names its registries (section 4).
- A **registry pack** is a pack published to a registry as an npm package (spec §6.4). A **library pack** is a package that holds a library packs share (section 5); it has no document.

## 2. A pack on a registry

A registry pack is a CompMD pack as spec §6.4 says: an npm package whose `package.json` is its manifest (F022 when `md.export` disagrees with it), whose document the `compmd` field names, and whose integrity is the hash the registry publishes for its tarball. In addition:

- **Its name is scoped**, `@scope/name`, as every pack's is; `@compmd/*` is reserved. No prefix is required: an import always names a pack in full, and the field and keyword below say what it is. A scope that publishes a library and a pack of the same name MAY prefix the pack's with `compmd-` (`@acme/charts`, a React library, beside `@acme/compmd-charts`, its pack).
- **It says it is a pack**, in its `package.json`:
  - the `compmd` field SHOULD be written, even when it names the default document (`"compmd": "index.comp.md"`). A registry keeps every field in a version's manifest, so a tool can tell a pack from other packages without its tarball. A host still reads a package without the field as spec §6.4 says.
  - its `keywords` SHOULD include `compmd-pack`, which a registry's search indexes, so a search for `keywords:compmd-pack` lists packs. Only packs use this keyword; packages that help build packs use `compmd`.
- **Each built module SHOULD be one file**, with no relative imports, so it loads where it is passed as text (spec §6.4). Its imports are only its binding's runtime and the libraries its pack names in `peerDependencies` (spec §7.6); every other library it uses is bundled in. A pack whose modules import files of their own is still a pack, but a host that can't load them renders those components' fallbacks with W121.
- **Its `dependencies` are never installed** by a host. What its components use is bundled into their modules, and the packs it imports are its document's `md.import`, which the host resolves (section 3). `dependencies` MAY still list what its build used.
- **Its `files`** SHOULD leave out what a host never reads (sources, tests), so its tarball is small. A host reads only `package.json`, the pack's documents and the files they import.
- **Its size** counts against the host's cap, if the host sets one: the version's unpacked size, which the registry's metadata states (section 3).

## 3. How a host reads a registry pack

A host that fetches a document's packs from a registry resolves them before processing, as it fetches the document's data:

1. **Which packs.** The packs the document imports that the host doesn't hold, and then those the fetched packs' documents import, until none is missing. A pack's imports go through the same host's registries and policy as the document's: a pack can't name a registry.
2. **Which version.** A range resolves to the newest version in the registry's list that matches it (spec §6.1). None matching is F026.
3. **Policy first.** Before anything is downloaded, the host applies its configuration (spec §6.4): a pack it doesn't allow or denies is F026; so is a version whose unpacked size, from the registry's metadata, exceeds its cap.
4. **The tarball, checked.** The host downloads the version's tarball and MUST check it against the integrity the registry's metadata gives for it (`dist.integrity`) and, when the import pins one, against the import's `integrity` (F035 when either doesn't match). Nothing in the pack is used that the tarball's hash doesn't cover.
5. **Its files.** The tarball is a gzipped tar of the package under `package/`. The host reads `package.json` from it (never from the registry's metadata alone), the document its `compmd` field names, and the files the pack's documents import. A built module is loaded from these files, by its place in the pack (spec §6.4).

A version's tarball never changes, so a host MAY keep what it fetched by its integrity, and serve a pinned import from that cache without fetching. A host MAY keep a pack's version list for as long as the registry says it may be cached.

## 4. Who fetches

Which registries a host uses is its configuration, by scope with a default for every other scope, as npm's `.npmrc` does. A document never names a registry, and neither does a pack, so neither can make a host or a reader's browser contact a server of its choosing. Without a registry in its configuration, an import of a pack the host doesn't have is F026.

- **By default, the host fetches**, itself or through its own proxy or cache, so the registry learns nothing about who reads which document (spec §6.4, §15 *Privacy*).
- **The reader's browser MAY fetch directly**, from the registries the host's configuration names, when that configuration says so; a host with no server, such as a static site, may need to. The registry then learns the reader's address, the time, and the names and versions of the packs the document imports, never the document or what it shows. The host SHOULD tell its readers. Such requests MUST send no cookies or other credentials, and carry nothing from the document but the names and versions of its imports. A document can never choose this; only the host's configuration can.

## 5. Library packs

A library pack holds one library that packs share, so a sandbox loads one copy instead of each pack bundling its own: worth it for large libraries several packs use (three.js, KaTeX).

- **It says it is one** with an object in the `compmd` field: `"compmd": { "library": "dist/index.js" }`, the path of its one module file. It has no document and no components. Its name is scoped, like a pack's, and SHOULD say what it wraps, with the library's version as its own (`@kit/three@0.170.0` is three.js 0.170.0).
- **Its module is one file**, an ES module whose only imports are the runtimes and other libraries by name, never by relative path.
- **A pack uses it** by naming it in `peerDependencies` (`"@kit/three": "^0.170"`) and importing it by that name in its components' code. A host that offers no library of that name (spec §6.3) resolves it from its registries like a pack (section 3), under the same policy.
- **One version per document.** For each library, the host uses the newest version that matches every range the document's packs ask for. When no version matches them all, each range gets its own, unless the library says it can't have two copies in one sandbox: `"compmd": { "library": …, "single": true }`. Then the pack asking for the second version gets W121 on its import, naming the library, both ranges and the version the document has (spec §6.4); its components that don't use the library still render.
- **It runs at the level of the pack that imports it**, in that pack's sandbox, and adds no trust of its own (spec §15).
