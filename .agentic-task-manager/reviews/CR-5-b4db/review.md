+++
id = "01a1eda9-4c56-724b-a6a6-17863908b4db"
number = 5
title = "The Mermaid page test says which diagram stalled"
status = "approved"
tasks = ["~CMD-248"]
author = "claude:79612601"
reviewers = []
created = 2026-11-18T07:57:37Z
updated = 2026-11-18T08:05:27Z
+++

The Mermaid pack's page test, when a diagram never draws, says which one and what the page said. Nothing is weakened: the same waits, the same assertions.

## What to check

- `compmd-packs/mermaid/test/page.test.ts`: `frameWith` catches the poll's timeout and throws an error naming the selector, the page's frames by URL, and the log `open` gathers: every page error and console message, with its source URL. The `problems` the first test asserts are empty are as before.
- Checked by renaming a label for a run: the error read "No frame has `[role="img"][aria-label="Idea, then plan, then nothing"] svg` after 30 s. The page's 7 frames: … What it logged: …".

## The failure itself

It passed on CI again on df13810, in WebKit in 4.7 s, on the first run whose WebKit was downloaded rather than restored from the cache. The cause isn't proven; the likeliest is WebKit and its libraries restored from a cache another runner image saved. The cache's changes are in b13b195 and df13810, outside this task.
