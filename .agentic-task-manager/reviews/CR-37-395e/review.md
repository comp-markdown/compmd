+++
id = "01a1fb1e-206a-7149-9886-4acc2eb5395e"
number = 37
title = "What compmd definitions need for the no-code components"
status = "approved"
tasks = ["CMD-359"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T22:40:18Z
updated = 2026-11-20T22:47:16Z
+++

The no-code components stop at the same few gaps in the `compmd` definition language. This design closes them with seven decisions, each tried on its component's own example. It answers the questions blocking CMD-314 (link-card), CMD-315 (gallery), CMD-316 (figure), CMD-317 (badge) and CMD-348 (columns), and covers CMD-318 (sticky-note).

## What to decide

1. A destination starting with a param resolves against the caller (link-card's `href`, figure's `src`, gallery's image fields). A `url` type is rejected.
2. `md.fallback` in a body, and interpolation in image alt text (figure's caption as its alt, badge's text).
3. `stretch-link`, a box flag: its first link covers the box (link-card), with W124 when there's no link.
4. An inline invocation is one inline box, styled by its fence (badge). An inline `:box` is rejected.
5. `md.slots.<name>` in a definition's expressions (columns). A `count` param is rejected.
6. `shadow` (tokens) and `rotate` (−15° to 15°) as common params (sticky-note). Pack stylesheets are rejected, for security.
7. gallery needs nothing more than decision 1.

*Gaps left* has two to know about: a definition can't report its own diagnostic (columns' `wide-middle` with no middle slot), and the processor keeps a pack's relative destinations as written (found by a probe), which decision 1 makes the expanded tree's to settle.

The spec text, the processor and the packs wait for your go on this.
