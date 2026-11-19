+++
id = "01a1f30c-6298-72e2-960b-b2caf23680bf"
number = 15
title = "A block at a time, those on screen first"
status = "open"
tasks = ["~CMD-259"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T09:03:57Z
updated = 2026-11-19T09:03:57Z
+++

The renderer highlights a document's code blocks one at a time, those on screen first, and a page has drawn once they are: the two answers on CMD-259.

## What to check

- **The queue** (`npm-packages/render/src/highlight-queue.ts`): one per document. A block is asked for a task after the last block's tokens are drawn (a `MessageChannel` message, `setTimeout` without one); the first block on screen, as an `IntersectionObserver` last said, else the first asked for. A block asked for again replaces what it waited for; one whose view is destroyed is dropped (`dropHighlight`); tokens for a block asked for again meanwhile aren't drawn. No layout is read.
- **The contract** is unchanged: a highlighter still gives `(code, language) => tokens`. `drawTokens` stays in `highlight.ts`.
- **Settling:** `CodeBlockView` gives its highlighting to the page's `busy`, so `page.drawn()`, a frame's `drawn` and a view's `settled` wait for it (within `BUSY_MS`, as a component's drawing). The READMEs and the sandbox's design (decision 8) say so.
- **Tests:** `highlight.test.ts` (one block at a time, on screen first with a stubbed observer, a removed block never asked for, `drawn()` waiting for tokens, and the earlier cases through `drawn()`); `code.test.ts` awaits `drawn()`; in `view.test.ts`, a host-mode view highlighted once it has settled, light and dark, Chrome and WebKit. `cargo make ci` passes.

## Measured

`spikes/highlight-perf/README.md`, *A block at a time*: in WebKit with Oniguruma, opening 84 ms blocked (longest frame 134 ms) and the skim 8 ms, against 253 ms and 151 ms before; the spike's best. With the JavaScript engine no better, its opening possibly worse (1.7 s against 1.2 s, runs that varied a lot): one block's first compile can't be split, and the blocks' compiles no longer share a frame.
