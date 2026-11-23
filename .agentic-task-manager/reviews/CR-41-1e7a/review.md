+++
id = "01a1fe25-af46-764a-8ba4-04413b521e7a"
number = 41
title = "Sizing: @compmd/pack fits what a component draws, and shows its box"
status = "approved"
tasks = ["CMD-356"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T12:47:24Z
updated = 2026-11-21T13:27:23Z
+++

[Sizing a component in its box](docs/sizing-components.md), *What changes*, `@compmd/pack`: the last of the four sizing tasks.

## What to check

- **`fitBox(element, fit)` and `fillBox(element)`** (`pack/src/fit.ts`, exported from `@compmd/pack/component`). `fitBox` gives an SVG, `img`, `canvas` or `video` the box's size: an SVG by `preserveAspectRatio`, the others by CSS's `object-fit` and `object-position`. An SVG with a width and height and no `viewBox` gets one, or it couldn't scale. `fillBox` is `contain: size` with CSS's 300 by 150.
- **`draw()`** fits a drawing that is one SVG, `img` or `canvas`; its own `div` is the box's size; `render` gets `fit`, and runs again when it changes. A drawing of several elements, or text, flows as before.
- **`mountForTest`** takes `fit` and has `setFit`. It gives them as the box's `object-fit` and `object-position`, as a document does.
- **Show boxes** on the test page: a CSS `outline` on each use's box, which moves nothing. Its browser test checks the box's rectangle is the same with it on.
- **The leaf template** now draws its value as an SVG fitted with `fitBox`, 12rem wide by its signature fence (`{width=12rem max-width=100%}`), and its demo has a use with `width`, one with `height` and one with `object-fit=cover`. The Developer's Guide's chapter 13 shows the same files, so it changed with it; chapters 14 and 17 name the helpers and `setFit`.
- **Mermaid and wireframe**: each demo has a sized use, and both packs' page tests pass in Chrome and WebKit.

## Three decisions to check

1. **`fitBox` keeps a drawing's `max-width`.** Mermaid's SVG gives its natural size that way. Without it, a two-node diagram would grow to the column's width. So a Mermaid diagram with no size looks as it did, and one in a sized box is fitted in it.
2. **A drawing narrower than its box is placed by `object-position`, so centred by default.** A Mermaid diagram narrower than its column was at the start before, and is centred now, which is what the spec's default says. `object-position="center start"` puts it back at the start. This is a visible change for existing documents.
3. **The renderer template stays text that flows,** so `object-fit=cover` has nothing to act on there. Its demo has a use with a `width` and one with a `height` that scrolls. The task asked for all three uses in both templates' demos.

## Also changed

`create-pack`'s test *makes each use's frame as tall as what it shows: neither cut short nor taller* is now *cuts no use short*: since CMD-355 a frame's body is the frame's height, so the test can't see a frame that is too tall. That a box is the size its component needs is `@compmd/sandbox`'s comparison test.

The template uses `fitBox`, which the released `@compmd/pack` 0.3.0 doesn't have: the template's pinned version moves with the next release, as it always does.

**A fix to CMD-355's test, in this commit:** `npm-packages/sandbox/test/sizing.test.ts`'s `row` case failed twice under ci's load. The boxes matched (304×154 both); the framed component's own report was stale, 1×1, because the row's third use wraps below the test page's small viewport, and Chrome pauses a frame's observers off screen. The comparison now scrolls a framed use that doesn't match yet into view, so its component can report. The layout itself was right throughout.
