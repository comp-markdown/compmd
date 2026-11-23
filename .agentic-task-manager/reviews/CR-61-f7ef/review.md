+++
id = "01a20401-0b6d-70ff-b2ce-071490c9f7ef"
number = 61
title = "grid-auto-rows: every row of a grid as tall as the tallest"
status = "approved"
tasks = ["CMD-369"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T16:05:07Z
updated = 2026-11-22T17:15:23Z
+++

CMD-369, from CR-50: `grid-auto-rows` on `box` and `grid`, so every row of cards is as tall as the tallest, however many rows `min` makes.

## What to check
- **The spec:** §9's tables (`box` and `grid` take it), `box`'s signature as quoted, a bullet *Rows of one height* with `::: grid {min=14rem grid-auto-rows=1fr}`, §9.4's tracks; the changelog; `llms.txt`.
- **The processor:** `core.comp.md`: `grid-auto-rows: tracks?` on `box`, and on `grid`, passed to its box; `params.json`.
- **Conformance:** `expand/grid-auto-rows`: a value, a breakpoint, `box`'s own, and a value that isn't tracks (F009, treated as absent).
- **The renderer:** `styles.ts`'s `LAYOUT`; `test/browser/grid-rows.test.ts` in Chrome and WebKit: four cards, two to a row, one height with `1fr`, each row its own without it.
- **The Author's Guide:** chapter 9's table, and an example under `grid`.
