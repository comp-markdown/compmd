+++
id = "01a1f9f9-495f-7382-af67-556e44d45934"
number = 29
title = "Wrapping an npm package as a pack: the design"
status = "approved"
tasks = ["CMD-8"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T17:20:26Z
updated = 2026-11-20T20:31:30Z
+++

Round 5: [sizing a component in its box](docs/sizing-components.md) reworked, as asked in conversation: no open gaps, positioning and cover and stretch in scope, and a prototype behind it.

What to check:

- **Decision 1:** `object-fit` (`contain`, `cover`, `fill`) and `object-position` (`start`, `center`, `end`, block then inline) as common params, written without a breakpoint, reaching the component as `context.fit`. CSS's names, by the spec's rule; `fill` here is CSS's stretch, beside the flag `fill`.
- **Decision 2, the rule that closes the gaps:** a use is laid out the same framed as in the page. Grid rows, rows, inline uses and uses with no size need no rule of their own: CSS decides them in the page, and the frame must match.
- **Decision 3:** a component's natural size is CSS's (300 wide with only a ratio, 300 by 150 with none); a drawing sized in script contains its size.
- **Decisions 4 to 6** are round 4's 3 to 5: box defaults on the signature fence (now also where a pack sets `object-fit`), the common params' names as F013, padding and border.
- **The mechanism,** and **the prototype** (`spikes/wrap-npm-package/sizing/`, `pnpm --dir spikes/wrap-npm-package sizing`): 28 uses drawn in the page and framed are the same in Chrome and WebKit, on screen and off it, and the grid rows again after they shrink. The spike's README says what it took (finding 14).
- **What the prototype doesn't run** is listed, each a browser test of the sandbox's task: right-to-left, `scroll`, an `img` or `canvas` drawing, and a script-sized drawing that doesn't contain its size.
- **Conformance:** two more new cases, for `object-fit` and `object-position`; the rest as round 4.
- **What changes** now names `@compmd/render` (the box a grid, `context.fit`) and the sandbox protocol's version 3.
