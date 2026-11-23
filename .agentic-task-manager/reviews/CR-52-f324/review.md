+++
id = "01a1fede-f29f-714f-8961-b430063ff324"
number = 52
title = "The sticky-note pack, and W119 for a box with nothing in its flow"
status = "approved"
tasks = ["CMD-318"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T16:09:46Z
updated = 2026-11-21T16:47:54Z
+++

The sticky-note pack, `@comp-markdown/sticky-note` (item 7 of [the first components](docs/first-components.md)), and the spec change it needed, which you chose in conversation: W119 relaxed.

## What to check
- **The spec (§9.2, §10.2):** a box that places blocks takes its height from the blocks left in its flow, as an article with a note pinned on it does; W119 is for a box whose blocks are all placed, with no `height`, `aspect-ratio` or `fill`. The changelog, `llms.txt`, the Author's Guide (chapters 11 and 20). The processor's check (`expand/layout.rs`, `flows`), and the conformance case `expand/positions-W119`, whose first box, sized by its paragraph, now has no warning.
- `index.comp.md`: the fence `{padding=m background=accent shadow=m rotate=-2deg width=14rem max-width=100%}`, the default slot.
- `demo.comp.md`: a note in the flow, one pinned over an article from `sm` up (a breakpoint follows its box's width, and the demo's column is narrower than `md`), and the box changed by the use.
- Tests: the expanded tree (the fence's params, the use's insets after them, the use's tilt and tint, F009 past 15°, the plain reader, the demo); `test/page.test.ts` in Chrome and WebKit: tilted with a shadow, pinned over its box's end when wide, in the flow after the text and inside its box on a phone, and printed tilted with its shadow.
