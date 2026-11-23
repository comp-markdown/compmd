+++
id = "01a1feaf-8c3d-716d-b4c1-8244f5efa3c4"
number = 48
title = "The rating pack: a value as stars"
status = "approved"
tasks = ["CMD-320"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T15:17:59Z
updated = 2026-11-21T15:29:25Z
+++

The rating pack, `@comp-markdown/rating` (item 9 of [the first components](docs/first-components.md)): a value as stars, half stars included, read-only, drawn as SVG in the strict sandbox.

## What to check
- `compmd-packs/rating/index.comp.md`: the signature, `value: number` and `of: number = 5`, in a box 7.5rem wide (`max-width=100%`).
- `src/rating.ts`: the stars, a half star cut by a `clipPath`; the value to the nearest half between 0 and `of`, at most 20 stars; the text alternative from the fallback, else "Rated X of Y"; the drawing fits its box.
- `demo.comp.md`: each param and use as a `{render=both}` example (a block, inline in a sentence, a half star, `of`, a size).
- Tests: `src/rating.test.ts` (the stars and their label); `test/page.test.ts` in Chrome and WebKit, every example drawn and labelled, nothing refused or reported, and printed.
- The README, `versions.json` and the workspace entry.
