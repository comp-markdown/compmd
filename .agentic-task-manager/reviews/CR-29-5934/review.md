+++
id = "01a1f9f9-495f-7382-af67-556e44d45934"
number = 29
title = "Wrapping an npm package as a pack: the design"
status = "open"
tasks = ["CMD-8"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T17:20:26Z
updated = 2026-11-20T18:59:35Z
+++

Round 2: the spike is in the project, `spikes/wrap-npm-package/`, and the design takes what it found.

The four packages are now packs written by hand as `--from` would write them, each with its `compmd.wrap.json`, glue, generated signature and demo. `try.ts` builds them as `compmd-pack build` does and opens each demo in strict frames in Chrome and WebKit (`pnpm --dir spikes/wrap-npm-package try`); `results.json` and `screenshots/` are its last run. react-qr-code, layerchart's `BarChart` and qrcode draw in both; model-viewer shows its fallback with W118, the frame refusing its model's fetch and its WebAssembly.

What to check:

- **The spike's README, *What it found*:** ten findings, the first six changing the design.
- **Decision 5:** the wrap's settings move to `compmd.wrap.json`, not published, since `package.json`'s `compmd` field is the pack's document already (registry companion, sections 2 and 5). "Own declaration" now means declared in the package's own files.
- **Decision 6, two rows:** CSS colours map to `color` (spec §9.4); a prop named like a common param (`height`, `width`, `background`, …) is the box's, never a param: layerchart's `height` never reached the component.
- **Decision 3:** the glue mounts in a box of its own; a web component's glue passes its `error` event to `fail`.
- **Decision 10:** Svelte's CSS injected into the module.
- **"In use"** now matches the spike's react-qr-code pack.
- **Gaps:** G6 (the processor accepts an optional enum; no conformance case yet) and a new G7: the `generated` flag in the fence's info string, which the spec doesn't define. A spec line, or a comment before the fence instead: your call.
- The workspace now lists `spikes/wrap-npm-package/*`, and the lockfile has the four packages; the packs have no build, lint or test script, so the workspace's runs skip them.
