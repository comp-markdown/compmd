+++
id = "01a2097f-3f7a-75bc-ab55-030ad87de25b"
number = 77
title = "Design: gradients in background"
status = "approved"
tasks = ["CMD-387"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T17:41:03Z
updated = 2026-11-23T17:49:17Z
+++

The design for gradients in `background` (`docs/background-gradients.md`), to review before the spec text and the code.

## To decide
- **Decision 4, proposed: an angle is logical**, mirrored in right-to-left text, as the `to` words are. It came from trying the sticky note's corner: a cut made with `to block-start inline-start` follows the box's diagonal, so it isn't 45° on a note taller than it is wide, and doesn't meet the flap. `315deg` is 45° on any box. The cost: a CSS angle copied into a right-to-left document points the other way.

## To check
- The grammar, a subset of CSS's: linear and radial (and repeating) only, at least two stops, no colour hints or interpolation spaces, `at` with one or two values.
- What is refused (F009): any other function (`url()`, `var()`, `calc()`, …), quotes, `;`, braces, and mixed logical and physical sides.
- The renderer's two rules (left to right, and `:dir(rtl)`), and W126 not checking gradients.
- The examples: the note's corner (and the thin shadow left past its cut, a gap), a fade, stripes, a spotlight.

After this is approved: the spec's §9.2 and §9.4, the changelog and the conformance cases (this task), then the code (CMD-388), then the note's corner (CMD-386, a new round of CR-76).
