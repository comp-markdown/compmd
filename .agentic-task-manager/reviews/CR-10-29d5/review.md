+++
id = "01a1f2cd-29ca-7647-ad57-f6513e9729d5"
number = 10
title = "One highlighter for a page’s views"
status = "approved"
tasks = ["~CMD-256"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T07:54:54Z
updated = 2026-11-19T08:13:09Z
+++

In host mode every view of a page shares one highlighter, so Shiki is made once and each grammar loads once.

## What to check

- **`framesHighlight`** (`npm-packages/sandbox/src/from-frames.ts`): the highlighters by the URL of the `highlight.js` they load, kept for the page; a failed load is forgotten, so the next block tries again. `load` is a parameter for the tests.
- **`shikiHighlight`'s docs** say a host that renders many pages itself makes one and shares it; the sandbox's README says views share one.
- **Frame mode** is unchanged: a frame is a page of its own, and its runtime already has one highlighter.
- **Tests:** `from-frames.test.ts` (one load for two views of one place, one per place, a load again after a failure); in `view.test.ts`, two host-mode views highlighted, light and dark, in Chrome and WebKit.

## Measured

On the spike's page with the sandbox's own highlighter (`?sandbox=per-view|shared`, `spikes/highlight-perf/README.md`): in WebKit the skim's blocked time goes from 4.2 s to 0.8 s and the revisit's from 3.5 s to 0; Chromium's skim from 284 ms to 67 ms. The opening's first long frame is the JavaScript engine's compile (CMD-257).

It builds on CMD-251 (CR-9), whose contract it uses.
