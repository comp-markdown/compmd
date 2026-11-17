# Any npm package, as published (proposed)

A pack's component that draws a chart wants `import * as d3 from "d3"`: the library as its authors publish it. Today it
can't. A frame loads one checked module file whose imports are names it knows (the runtimes, a library the host offers,
a library pack), and a package as published is rarely that: d3 7.9's module entry imports 30 other packages, each of
many files, and its one-file build is a classic script. So a library reaches a frame only as a *library pack*, the
library built into one module and published again under another name ([packs from a registry](packs-from-a-registry.md),
R9). This decides how any npm package, as its authors publish it, becomes something a pack imports by its own name.

## Decisions

1. **Which packages: ES modules only.** A package whose `exports` (with the conditions `browser`, `import`, `default`, in
   that order) or `module` field names an ES module, and every package it imports likewise. A package that is CommonJS
   only (`main` alone, `require` in its code) is refused, with W121 on the import that needs it, naming the package: a
   browser can't run it, and converting it would be guessing at Node's semantics.

2. **How a pack names one: in `peerDependencies`, as a library pack.** `"d3": "^7.9"` beside `"@kit/mermaid": "^1"`, and
   its components import it by that name. The same rules as library packs (R9): one version per document, the newest
   every range allows; `single` when two copies can't share a frame (`"compmd": { "single": ["d3"] }` in the pack's
   manifest, as the package itself can't say it); a range nothing satisfies is W121. A library pack stays a library
   pack: one of these that happens to be one built module.

3. **Fetching: the registry resolves the whole tree.** `@compmd/registry` gets the package as it gets a pack (its
   versions, its tarball, checked against the integrity npm publishes), then each package its module graph imports,
   one version per range, as npm would dedupe them, each checked the same way. The host's policy applies to every
   package of the tree: `deny`, the size cap (for the tree, not each package), and `imports: listed` (a listed root
   brings its tree). A first visit fetches the tree once (d3: 31 tarballs); later ones, nothing but expired versions.

4. **Where it becomes loadable: bundled once, in the compile frame.** The tree's files go to the compile frame, which
   bundles them into one module with Rollup's browser build (plain JavaScript, so no WebAssembly, which the strict
   policy forbids): Node's resolution (`exports`, `module`, relative and bare imports between the tree's packages),
   cycles, and one module out, whose imports are only the runtimes. The frames then load it as a library pack's
   module, a `blob:` module named by its URL, and nothing in them changes. The bundle is kept by the tree's integrities
   in the compiled-code cache, so a package bundles once per browser, not per visit.

   | | A. A loader in every frame (R7's K2) | B. Bundled in the compile frame (recommended) | C. Bundled on the host's server |
   |---|---|---|---|
   | New code | a module loader in every frame (es-module-shims' JavaScript lexer, about 20 KB) | Rollup's browser build in the compile frame only (about 300 KB, loaded when a tree needs it) | a bundler on the server (F1) |
   | Cycles between modules | the loader's stand-ins | the bundler's own handling | the bundler's |
   | Per frame | resolves and links hundreds of files | loads one module | loads one module |
   | Static hosts | yes | yes | no |
   | Cached | each file | the bundle, by the tree's integrities | on the server |

   B keeps the frames as they are (their policy, their loading, what the sandbox's tests check), runs the bundler where
   untrusted text is already turned into code (the compile frame, sandboxed, no network), and pays once per browser.

5. **No code runs to bundle.** The bundler parses and links; it runs no package's code, scripts or plugins (`postinstall`
   and the like are ignored, as a tarball's are today). Only the tree's files take part, so a bundle holds nothing the
   registry didn't check.

6. **The spec.** §7.6 adds what a bare import may resolve to: a package a pack names in `peerDependencies`, as an ES
   module tree; *CompMD packs on a registry* says what such a package must be (decision 1) and how a host gets it
   (decisions 3 and 4). A processor that can't get or bundle it gives W121, as for any library it can't have.

## The spike first

What decides B: on a first visit, how long the tree takes to fetch and bundle, and how big the result is. Measured on a
laptop and a phone, cold and warm cache:

- d3 7.9 whole (`import * as d3 from "d3"`): 31 packages; the number of files, the bytes fetched, the bundle's size and
  time;
- a chart pack's usual part of it (`d3-scale`, `d3-shape`, `d3-axis`), to see what the tree costs a pack that uses
  little;
- Mermaid 11 as published (its ES module entry and its chunks), the large case.

If bundling takes more than a second on a phone for d3, the alternative to weigh again is C for hosts with a server.

## What it changes

- `@compmd/registry`: resolving a package's module tree, one version per range, and the policy over it.
- `@compmd/sandbox`: the compile frame bundles a tree (Rollup's browser build, loaded on demand); the bundle kept in the
  compiled-code cache; frames unchanged.
- The processor: `peerDependencies` naming npm packages, W121 for one it can't have.
- The spec's §7.6 and *CompMD packs on a registry*; `@compmd/pack`'s check accepting such `peerDependencies`.

## Open

- Whether a pack may also name a package in `dependencies`, its own copy, bundled with the pack rather than shared
  (proposed: no; a pack that wants its own copy bundles it in its build, as today).
- Whether the size cap counts the tree's files or the bundle (proposed: the files fetched, which is what the reader
  downloads).
