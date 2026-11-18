+++
id = "01a1ebb5-a711-7023-808f-7961c6012036"
number = 2
title = "Wireframes in the Author’s Guide"
status = "approved"
tasks = ["~CMD-36"]
author = "claude:e2fd71c2"
reviewers = []
created = 2026-11-17T22:51:52Z
updated = 2026-11-18T00:11:36Z
+++

The Author's Guide's chapter 17 gets a **Wireframes** section: the garden newsletter sketches its plot-booking screen with `@comp-markdown/wireframe`.

## What to check

- The section in `guides/authors/17-code-blocks-a-pack-draws.md`: the outline language in brief, `alt`, what a mistake shows (W118 with line and column), and `render=both`.
- The example `guides/authors/examples/code-blocks/booking.comp.md`, checked by the guide's tests (`registryPacks` now covers mermaid and wireframe) and by the pack's own test that its sketch is valid.

## Left for the release

No playground link yet: the playground gets packs from npm, so the link goes in once the pack is published at the end of the sprint.
