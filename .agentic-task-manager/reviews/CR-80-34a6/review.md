+++
id = "01a20a36-6075-7314-b8d3-ec5dd81d34a6"
number = 80
title = "Processor and renderer: gradients in background"
status = "approved"
tasks = ["CMD-388"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T21:01:05Z
updated = 2026-11-23T21:13:33Z
+++

Gradients in `background`, in the processor and the renderer, as the spec (CR-78) and the design (CR-77) say.

## The processor
- **The check:** a gradient form, `Form::Gradient`, in a module of its own, `values/forms/gradient.rs`, a child of `forms` so it uses its color, length and splitting helpers. It covers `linear-gradient`, `radial-gradient` and their `repeating-` forms, with:
  - an angle, or `to` with one or two sides of one frame;
  - a radial gradient's shape, size and `at` position;
  - two or more stops, each a color and up to two lengths.
- **`Kind::SURFACE`:** takes `color` and `gradient`, with an example of each, so F009's message names both. `conformance/params.json` is updated.
- **Tests:**
  - accepted and refused gradients;
  - each of `url(`, `var(`, `calc(`, `image(`, `;`, `{`, `}`, a quote and a backslash spliced in at every position of ten accepted gradients, each result refused. This takes the place of the property test the task named: the repository has no `proptest`, and its Rust guide keeps it for algebraic code;
  - two conformance cases: `validate/background-gradients`, every form accepted and no W126 for a gradient with a `color`, and `validate/background-gradient-refused`, six F009s.

## The renderer
- **`gradient.ts`:** `gradientFor(value, direction)` makes a gradient's logical sides physical, and mirrors its angle in right-to-left text (`calc(360deg - a)`). `followsDirection(value)` says whether it needs to.
- **`styles.ts`:**
  - A block's background is written for left-to-right text.
  - When one of its backgrounds follows the direction, `BlockCss.rtl` holds every one of its backgrounds for right-to-left text, at each breakpoint, so a plain color from `md` up still wins.
  - `flush` writes these as `.class:dir(rtl)` rules after the block's own.
- **Tests:** unit tests, and `test/browser/gradients.test.ts`, which checks in Chrome and WebKit that a box's drawn background equals the physical gradient it stands for, in both directions.

## Docs
The processor's `docs/param-values.md`, and the Author's Guide chapter 3's table of common params.

`cargo make ci` passes. Two earlier runs failed on things outside this change: CMD-389's edit bug, which these cases exposed, and a timeout while printing the PDFs under load, which passed alone.
