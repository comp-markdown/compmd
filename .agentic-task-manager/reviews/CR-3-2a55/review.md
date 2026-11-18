+++
id = "01a1ec07-9082-77c8-aa11-726a52702a55"
number = 3
title = "The booking screen in an example folder of its own"
status = "approved"
tasks = ["~CMD-36"]
author = "claude:79612601"
reviewers = []
created = 2026-11-18T00:21:21Z
updated = 2026-11-18T00:22:15Z
+++

A follow-up on CR-2: the booking screen's document broke the playground, and with it the books' PDFs.

## What went wrong

The playground makes one entry per document in `guides/*/examples/<folder>/`, named and identified by its folder. `booking.comp.md` was a second document in `code-blocks/`, so two entries were "Author's Guide: code blocks" with the same id. The playground's menu is keyed by name, so it threw `each_key_duplicate` and rendered nothing, and the website's PDF step timed out waiting for each example's picture.

## What to check

- `guides/authors/examples/wireframes/booking.comp.md`: the document moved, unchanged; chapter 17's snippet and the wireframe pack's test name its new place.
- `editor/src/documents.ts`: the playground leaves `wireframes/` out until the wireframe pack is on npm.
- `editor/src/documents.test.ts`: every example has a name and an id no other has. With the document put back in `code-blocks/`, it fails and names `guides/authors/code-blocks`.

The playground's tests (99), the guides' (262) and the wireframe pack's pass, and `pnpm --dir website build` prints both books and the spec.
