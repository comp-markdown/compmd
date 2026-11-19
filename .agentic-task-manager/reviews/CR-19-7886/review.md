+++
id = "01a1f4ea-defb-703c-adf7-610af8737886"
number = 19
title = "The first components: research and a ranked list"
status = "open"
tasks = ["CMD-10"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T17:46:35Z
updated = 2026-11-19T17:46:35Z
+++

The research for the first components: what 11 documentation frameworks ship, about 60 npm component packages (downloads, licences, sizes), and what CompMD has already, with a ranked first list of 12.

What to check:

- **The decisions:** ranking for posts, newsletters and notes as well as docs (1); definitions without code where they can be (3); the plain binding (4); only what runs in a strict frame (5).
- **The list:** its order, and whether anything is missing or shouldn't be there (a file tree, icons and an API field table are left out).
- **The packages chosen:** Observable Plot for charts, `@viz-js/viz` for Graphviz, `gridjs` for tables, `abcjs`, `qrcode`.
- **G4:** whether `tabs` and `details` go in `@compmd/core`, implemented by the processor (a spec change), or in a pack.
- **G1 to G3:** images, embeds and maps, which wait for designs of their own.
