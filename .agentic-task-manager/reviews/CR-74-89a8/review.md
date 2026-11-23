+++
id = "01a20920-84bf-76ec-ab0a-da7bf4b189a8"
number = 74
title = "Blog: installed packs without compmd.config.json"
status = "approved"
tasks = ["CMD-384"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T15:57:35Z
updated = 2026-11-23T15:59:26Z
+++

A blog without a `compmd.config.json` now gives its posts every pack it installs, as an app does (docs/blog-tool.md, decision 2; `npm-packages/vite/README.md`).

- **The fix:** `packsOf` (`npm-packages/blog/src/live.ts`) always reads the blog with `loadApp`; it no longer returns no packs when the file is missing.
- **What "installed" means:** as for any app, a pack is installed when it is a dependency in the blog's `package.json`, which is what `npm install` writes.
- **The docs:** the Author's Guide chapter 19 no longer tells writers to add `{}`. Installing the pack is enough, and `compmd.config.json` is only for more (ambient or trusted packs). The README's folder listing says the same.

## Tests
- `live.test.ts`: a blog with a no-code pack in `node_modules`, as its dependency, and no `compmd.config.json`. Its post imports the pack and builds with no warning, the component drawn. Before the fix it was `F026`.
- `cargo make ci` passes.
