+++
id = "01a1f9c3-335d-70a7-804a-1577c4ebc8c3"
number = 28
title = "The first packs: one per component, and WebAssembly in the frames again"
status = "open"
tasks = ["CMD-11"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T16:21:21Z
updated = 2026-11-20T16:21:21Z
+++

The first packs, as decided in the conversation (2026-11-20): one pack per component, and the frames compile WebAssembly again for graphviz and jq.

What to check:

- **Decision 10, one pack per component:** `@comp-markdown/<component>` in `compmd-packs/<component>/`, a version each; the embeds one pack per provider, sharing CMD-332's permissive frame code as a library; the basics stay `@compmd/basics` in the sandbox.
- **One thing I added, for you to confirm:** with about 25 packs, one release workflow for every pack, by its tag `<pack>-v<version>`, rather than a workflow each as the Mermaid and wireframe packs have (releases.md, decision 8). If you agree, it becomes part of the first pack task that releases (CMD-9's pack tool, or the first component).
- **Decision 11, WebAssembly again:** the frames' `'wasm-unsafe-eval'` comes back in CMD-324 (graphviz), whose description now says so with the files it touches; jq (CMD-326) now depends on it. WebAssembly in the frames' status says so.
- **Decision 9's example** names real packs now (`card`, `chart`, `music`, `embed-youtube`).
- **The tasks:** each component task lists its pack's folder (`compmd-packs/<name>/`; tabs, details and steps the sandbox).
