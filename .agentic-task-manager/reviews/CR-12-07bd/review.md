+++
id = "01a1f2d6-8037-7099-8967-15488c1007bd"
number = 12
title = "Settling reads the page’s links, not the whole page"
status = "open"
tasks = ["~CMD-255"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T08:05:06Z
updated = 2026-11-19T08:05:06Z
+++

A view settling no longer looks through the whole page for its stylesheets: it reads the page's links from a list the browser keeps up to date, one per document.

## What to check

- **`stylesheetsLoaded(document)`** (`npm-packages/sandbox/src/settling.ts`): the document's `getElementsByTagName("link")`, a live collection kept in a `WeakMap` per document; a link whose `rel` is exactly `stylesheet` (as the selector `link[rel="stylesheet"]` matched) and has no sheet yet is waited for, until it loads or fails. A link added after an earlier view settled is in the collection.
- **Fonts:** `document.fonts.ready` is already one promise for the document, so awaiting it costs a view nothing; unchanged.
- **What stays each view's own:** its drawing, its frames, its images and the still frame; `settled` means what it did.
- **Tests:** `settling.test.ts` (no whole-document query over 50 settles; a stylesheet added later is waited for; one that fails counts, other links don't). The browser tests of settling pass as before.

Not measured: what the selector cost per view in a large page; the change removes it rather than speeding it up.
