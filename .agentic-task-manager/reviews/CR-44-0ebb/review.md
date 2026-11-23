+++
id = "01a1fe73-6152-7596-9bee-7a652fe20ebb"
number = 44
title = "A pack’s own files in a page frame (frame mode)"
status = "approved"
tasks = ["CMD-364"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T14:12:16Z
updated = 2026-11-21T14:26:08Z
+++

A pack's own files in every frame, following CMD-362 (spec §12.3): a link or an image from a pack's definition names its file by its place, with `pack: true`.

## What changes

- **The packs' base URL, not a function:** a function can't cross `postMessage`, so the sandbox works with where the host serves the packs' files, ending in `/` (`/compmd/packs/`), and makes a file's URL from it (`packFileOf`). `found` gives it as `packs`: `packs/` under the bundle's origin, from the root of the page's own origin when it is on it, so the image gate takes such an image for the host's own. A host that serves the packs itself gives `packs` beside `base`.
- **The protocol:** `level` gains `packs`, as it has `schemes` (messages, `protocol.schema.json`, `docs/sandbox-protocol.md`). The frame end passes it only when it is text. Every frame gets it: the page frame (frame mode), each use's frame (`frameUse`), and the frames a frame makes for uses inside it.
- **The frame:** its page and its part regions get `packFile` from it, so a link to a pack's file has the same `href` as in host mode.
- **The images:** `@compmd/render`'s `imageSources` takes `packFile`, and lists a pack's image by its URL, or not at all without one. `PageImages` takes it too, so the host's page gives the frame the image it asks for by that URL, and nothing by the place itself.
- **Host mode** (`pages.ts`, the runtime's `mountUse`): `packFile` from the same base.

## Tests

`PageImages` with a pack's image; both protocol ends and the schema with `packs`; `found`'s `packs`. A browser test in Chrome and WebKit (`test/images.test.ts`): a pack's image shows in a page frame, from a file the host's page fetched, and in host mode, with the link to the pack's file the same in both.

## Not in this change

A registry pack's files (CMD-365), which waits on a decision.
