+++
id = "01a1fb16-d884-71ea-a0e5-301be5173e44"
number = 36
title = "Sizing: a component’s box is a grid, and it gets its fit"
status = "approved"
tasks = ["CMD-354"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T22:32:20Z
updated = 2026-11-20T22:45:17Z
+++

[Sizing a component in its box](docs/sizing-components.md), *What changes*, `@compmd/render`.

## What to check

- **The box** (`styles.ts`, base rules): `.compmd-component` is a grid of one column (`grid-template-columns: minmax(0, 1fr)`, `grid-auto-rows: minmax(0, auto)`), and an inline one (`span`) an inline grid, as the prototype's `.box`. Both rules are at one class (`:where(span)`), so `show` and `hide`, whose rules are one class and come later, still hide a box. This changes every host-language component's box on every page, so the website and the guides' tests ran in ci too.
- **`context.fit`** (`fit.ts`, `component.ts`): `{ mode, x, y }` from the box's `object-fit` and `object-position` (the values written without a breakpoint), the block position then the inline one, one value being both. In a right-to-left box the inline start is the right (`x: 1`). The direction is the nearest `dir` attribute, or `:dir(rtl)` for `dir="auto"`. It is read when the component mounts, once it is placed, and again on each update. A fit that is the same keeps its object, and one that changes is set before the `update` it comes with, as `scheme` is.
- **The contract**: `Fit` and `MountContext.fit`, with TSDoc. The React and Svelte bindings pass `compmd.fit`.
- **Browser tests**: the renderer gets a `browser` project (Chrome and WebKit, `@vitest/browser-playwright` and `playwright` at the sandbox's versions). `test/browser/sizing.test.ts` draws the prototype's in-page cases through `createPage`, with the sizes the prototype recorded, which are the same in both browsers. Text is measured against its own height, since fonts differ on Linux. Without the grid rules, 8 of them fail, among them WebKit's `aspect-ratio` box grown by its border.
- `mountForTest`'s `fit` option and `setFit` are CMD-356's. Two test contexts built by hand gained a `fit`, so they type-check.

## Not here

What a framed use does (the sizer, the natural size, `fit` with the part) is CMD-355. `draw()`, `fitBox` and `fillBox` are CMD-356.
