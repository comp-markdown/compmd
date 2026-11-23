+++
id = "01a1fe94-8085-74d2-a573-eeb82ca10d0c"
number = 47
title = "A registry pack’s images, from blob: URLs"
status = "approved"
tasks = ["CMD-365"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T14:48:27Z
updated = 2026-11-21T15:06:04Z
+++

A registry pack's images, as decided (a): `resolvePacks` gives them, checked, and the sandbox shows them from `blob:` URLs.

## Two things the decision led to

1. **Binary files are kept now.** The resolver kept a tarball's text files only (`textFiles`), and its cache text only, so a pack's PNG was dropped. `binaryFiles` keeps the others as bytes. `PackCache` gains `binaryFiles` and `keepBinary`, optional, so a host's own cache still works, giving text only. `memoryCache` and `defaultCache` keep them: Cache Storage holds them as base64 in JSON, under keys of their own. A cache that keeps binary files but has none for a version kept its text before it did, so the tarball is fetched again once.
2. **Only images, never through a link.** A `blob:` URL has the origin of the page that made it, so a pack's page or SVG opened from one would run with the host's page's origin. `images(place)` gives a file with an image type by its extension (`image-types.ts`), and nothing else. The image's `src` stays its URL under the packs' base, as CMD-364 gives it. The renderer rejects a `blob:` URL there anyway: it is one of the schemes a host can never name. What maps it to the blob is the host's image source, as it does for a host's own scheme:
   - host mode, `hostImages`: a held image from a `blob:` URL of it, made once;
   - frame mode, `PageImages`: the frame asks for the image by that URL, and the host's page gives the held `Blob` instead of fetching it.

   A link to a registry pack's file keeps its URL under the packs' base, which serves nothing for such a pack: it doesn't open. Said in the option's docs.

## The API

- `@compmd/registry`: `Resolved.images(place)`, by the place the expanded tree names (`@acme/kit@1.4.2/icons/star.svg`).
- `@compmd/sandbox`: `packImages`, beside `modules`. An app wires it as it wires the modules: `packImages: (place) => resolved?.images(place)` (`docs/vite.md`, the Vite README, the Mermaid pack's registry app).

## Tests

- `@kit/icons@1.0.0`, a fixture pack with an SVG, a PNG and an HTML page. Its images come with their types and bytes, and its page and document don't. The processor's tree names the image by that place.
- From the cache again, nothing fetched; and a cache that kept text only before is fetched from once.
- The tar split, and the caches' binary files, in memory and across two caches through Cache Storage.
- `hostImages`: a held image as one `blob:` URL; a held file that isn't an image is not given one.
- `PageImages`: a held image given with no fetch, a bundled pack's fetched.
- In Chrome and WebKit: a registry pack's image the host holds, which nothing serves, shows in frame mode and host mode.

## Docs

`docs/resolving-registry-packs.md`, decision 6 (binary files kept, images given); the READMEs of `registry` and `sandbox`.
