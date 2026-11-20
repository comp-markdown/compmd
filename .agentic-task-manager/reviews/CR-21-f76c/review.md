+++
id = "01a1f597-d1bf-7045-991e-3cca1132f76c"
number = 21
title = "The host’s code highlighter, and where the default Shiki runs"
status = "approved"
tasks = ["CMD-267"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T20:55:29Z
updated = 2026-11-19T21:58:23Z
+++

The design of the sandbox's `highlight` option, and where the default Shiki runs, measured on pages of 1, 10 and 40 frames in WebKit and Chromium (the spike's new `?framepage=N`).

What to check:

- **Decisions 1 to 6, 8:** the option beside `ownImages`; frames asking their parent (way B, and why not a module in the frames); `highlight` and `highlighted` in the sandbox protocol's version 2; a parent answering only for blocks it sent; the timeout keeping text; the host's page answering a block a task; no batching.
- **Decision 7, for you to decide:** Shiki moving to the host's page as the default. [Shiki as the default](#shiki-as-the-default) has three example pages, a table of the measured costs (blocked time, the last frame's first block, heap) for each browser, and what each way needs. In short: WebKit runs frames in the page's process, so ten frames block 2.8 s with a Shiki each and 0.4 s with the page's; forty, 14 s and 0.7 s. Desktop Chrome gives each frame a process, so today's way keeps the page's thread free there, at about 27 MB a frame. Moving it lets the frames drop `'wasm-unsafe-eval'`.
- **The threats,** and the caveats of the measurements (local files, one machine, TypeScript only, the host side without its queue).
