+++
id = "01a1fe4f-0d1e-72fd-8650-d5e9a46ec24b"
number = 42
title = "A pack’s own links and images reach the pack’s files"
status = "approved"
tasks = ["CMD-362"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T13:32:35Z
updated = 2026-11-21T13:43:08Z
+++

A pack's own links and images reach the pack's files, as decided: the pack's place, marked with a field on the node. **The spec text needs your go before it is pushed.**

## The spec

- **§12.3, *Relative destinations*:** a link's or image's relative destination is resolved in the expanded tree. One that leads to a file of the root folder is written relative to the processed document (`./icons/star.svg` in `parts/cards.comp.md`, imported by `posts/harbour.comp.md`, is `../parts/icons/star.svg`). One that leads to a pack's file is its place, with `pack: true`. A renderer asks its host for the URL, and a host serves every file of its packs.
- **§8.6:** "making a pack's files reachable is the processor's job" now says how, pointing at §12.3.
- The changelog under *Unreleased*, and `spec/llms.txt`.

## The processor

- `expand/destinations.rs`: the resolution, a pure function with its tests. It keeps a URL with a scheme, one from the root of its host (`/x`), a fragment or a query alone, and one written in the processed document itself. A path that leaves its root is F026, kept as written.
- **Which document a destination resolves against** (§8.6, from CMD-360): an `Env` now knows, for each name whose value came from a caller, the document that value follows (`bases`). A body's params the caller gave, and an `each` name over one, map to the caller's. So a param a pack's `teaser` passes on to its own `link-card` (`href=(href)`) still leads to the document that called `teaser`, not into the pack. A param left to its default follows the definition.
- `cel/variables.rs`: the variables an expression reads, which that needs.
- Link and Image have `pack`, serialized only when true.

## Conformance

`expand/definition-own-files`: an ambient pack with a literal image and link, a param passed through another definition, an `each` over a param, and a path past the pack's root (F026); a file the document imports, and the document's own image, kept. `compmd-tree.schema.json`: `pack` is `true` on a link or image of the expanded tree, and never in the parse tree.

## The renderer and the sandbox

- `@compmd/render`: the `packFile(place)` option, as `wikiLink` is. With no answer, a link shows its text and an image its alt.
- `@compmd/sandbox`, host mode: `found` gives `packFile` from the origin, `packs/<place>`, where the bundle and the host's store serve packs. On the page's own origin it is a path from the root, so the image gate takes a pack's image for the host's own and never asks the reader about it.

## Split off, as agreed

- **CMD-364:** frame mode (the frame gets the mapping, and `PageImages` lists pack images by it).
- **CMD-365:** the files of a pack fetched from a registry while a document is open, which nothing serves yet.

## The guides

Author's Guide chapter 12 (a definition's own files, imported or from a pack), Developer's Guide chapter 7 (`packFile`).
