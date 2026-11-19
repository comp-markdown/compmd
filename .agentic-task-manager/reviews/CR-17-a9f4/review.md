+++
id = "01a1f366-a672-7145-90dd-d970aeefa9f4"
number = 17
title = "Before 0.8.0: the landing page, the guides and the link check"
status = "open"
tasks = ["~CMD-264"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T10:42:33Z
updated = 2026-11-19T10:42:33Z
+++

What readers see, checked against what changed since 0.7.1, and fixed as agreed (items 1, 2, 4, 5, 6 and 8; 3 is in CMD-263; 7, 9 and 10 skipped).

## What to check

- **The landing page** (`website/src/App.svelte`): the three links broken by the repository's move (the spec's changelog, the processor, its incremental design) point to `spec/` and `crates/processor/`; its links to the repository's files are written out in full, so the link check reads them; the npm list adds the blog tool and what a host serves, the crates list `compmd-pack`.
- **The Developer's Guide:**
  - chapter 5: `view.lineHeights()`, with a diff's two sides kept level;
  - chapter 7, *Code blocks*: a block at a time, those on screen first, `settled()` waiting for them; the two engines and `'wasm-unsafe-eval'`; and in the `createPage` options, `shikiHighlight({ webAssembly: true })`, one for every page;
  - chapter 8: the frames may compile WebAssembly, which reaches only what their scripts reach;
  - chapter 11: many views in one page share a stylesheet and a highlighter; `strict.html` may compile WebAssembly; `highlight.js` with both engines; `settled()` waits for highlighting; `cspFor`'s `'wasm-unsafe-eval'`.
- **The link check** (`lychee.toml`, `lychee-inputs.txt`): `.svelte` read too, and the website's and the playground's sources checked; the site's analytics left out, which a check would count as a visit; `docs/ci.md` decision 14 says so. The site's own relative links (`spec/`, `guides/`) are untouched, and nothing new reaches compmd.dev. A scratch `.svelte` with the old links fails; the repository passes.

## Checks

`cargo make links`, `cargo make site` and the guides' tests pass.

## Left for the user

The playground in Safari and Chrome: code blocks highlighted in both modes and both schemes, nothing slower.
