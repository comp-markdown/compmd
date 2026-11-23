+++
id = "01a20429-a10c-7780-9e9b-14f2e87543c1"
number = 63
title = "What CompMD puts in a page, under a strict style-src"
status = "approved"
tasks = ["CMD-375"]
author = "claude:79612601"
reviewers = []
created = 2026-11-22T16:49:26Z
updated = 2026-11-22T17:16:17Z
+++

CMD-375: what CompMD puts in a host's own page, under `style-src 'self'` (no `'unsafe-inline'`).

## What to check
- **`npm-packages/sandbox/test/strict-style.test.ts`**, in Chrome and WebKit: the test page gets the policy as a `<meta>`, and every `securitypolicyviolation` is recorded. In host mode a tree with inline and display math (Temml), highlighted code (Shiki), a grid box with spacing, border, font and colour params, an example and a strict component in its frame renders, and nothing is refused. A second test shows the policy is in force: an inline `style` attribute is refused.
- **The finding:** nothing to fix and no task to file. The boxes' rules are constructed stylesheets, and math and highlighted code are styled through the CSSOM; host mode uses `@compmd/render`'s page, so the renderer alone is covered too.
- **The Developer's Guide**, chapter 11: what a host's own policy needs (`frame-src` for the frames, `img-src` and `font-src` for images, fonts and packs' files), and that a pack-tool drawing brings its styles through the CSSOM.

## Verified
The sandbox's whole suite passes (61 files). The full `cargo make ci` wasn't run green: the tree has someone else's uncommitted changes to `compmd-packs/mermaid/test/strict-style*`, which fail, outside this commit.
