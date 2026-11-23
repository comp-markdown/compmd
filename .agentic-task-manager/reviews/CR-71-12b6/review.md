+++
id = "01a2061a-a6f2-7144-95cf-b1121b2612b6"
number = 71
title = "A live post rebases its relative URLs before rendering"
status = "approved"
tasks = ["CMD-381"]
author = "claude:79612601"
reviewers = []
created = 2026-11-23T01:52:19Z
updated = 2026-11-23T02:12:48Z
+++

A live post's images broke a moment after the page loaded: the blog's script renders the post again through the sandbox, which resolved its relative URLs against the post's page, a folder below the posts', so `first-packs/harbour.svg` asked for `/blog/first-packs/first-packs/harbour.svg`.

## What to check
- **`rebase.ts`** (new): `rebasedTree` puts `../` before the expanded tree's relative URLs (images, links, link definitions, fonts' files, in examples too), keeping a pack's files and params, which are data. `isRelative` and `rebased` moved here from `render.ts`, which the static HTML's rebase still uses.
- **`client.ts`** renders the rebased tree, first and after every input change. The `../` is `POST_FOLDER` (`client-data.ts`), which the build's static rebase uses too.

## Checked
`rebase.test.ts`. On a dev server, `/blog/first-packs/`: the live render takes the static post's place and all six images load. `cargo make ci` passes.
