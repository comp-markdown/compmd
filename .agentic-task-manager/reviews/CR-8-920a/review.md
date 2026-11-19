+++
id = "01a1f2ae-e64a-7756-8bf6-93781ac1920a"
number = 8
title = "One stylesheet per document, and the scheme read on need"
status = "approved"
tasks = ["~CMD-253"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T07:21:50Z
updated = 2026-11-19T07:27:35Z
+++

A document's pages share one stylesheet, and the colour scheme is read only when something needs it, so opening a page of text and code no longer makes the browser style the whole host document.

## What to check

- **The stylesheet** (`npm-packages/render/src/styles.ts`): `DocumentSheet`, one per document, adopted with the first `Stylesheet` and dropped with the last; `Stylesheet` keeps its API and is now one page's rules in it. Block classes come from the document's counter, never used again; a removed stylesheet's rules leave the sheet; `text` is still the base rules and the page's own, which the blog and the website's snapshots build from.
- **The scheme** (`npm-packages/render/src/scheme.ts`): read at the first `current`, again at `read()` only while something follows it, and its listeners only then.
- **The sandbox's view** (`npm-packages/sandbox/src/sandbox.ts`): `followScheme` reads only when the view has a frame, and runs again after a render, a splice and new code, when a use's frame may be new.
- **Security:** the three requirements agreed: names never reused in a document, a destroyed page's rules removed, the value guard (`safe`) unchanged. Frame mode is as before: a frame is a document of its own.
- **Tests:** `styles.test.ts`, `scheme.test.ts`, and in `view.test.ts` (Chrome and WebKit) two views styling their own blocks, and no style read for views of text and code. `identity.test.ts` compares pages with their block classes numbered as they come.

## Measured

`spikes/highlight-perf/README.md`, *A large host document*: 20,000 host elements, 100 pages opened and closed. WebKit, before: the document's sheets set 192 times, 200 styles read, 2.6 s blocked. After: 1, 0, and 0 blocked. Pages with styled blocks still write the sheet at each render (CMD-254).

## Not in it

Writing a rule at a time (CMD-254), shared settling (CMD-255), a `scheme` option and regions on the sandbox (held until measured).
