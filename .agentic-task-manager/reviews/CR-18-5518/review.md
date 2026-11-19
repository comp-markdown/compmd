+++
id = "01a1f4a1-67f1-70a4-b6e8-94081df25518"
number = 18
title = "Release 0.8.0"
status = "approved"
tasks = ["CMD-263"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T16:26:21Z
updated = 2026-11-19T16:26:38Z
+++

0.8.0 is released: the 11 npm packages at 0.8.0, the crates `compmd`, `compmd-pack`, `compmd-host` and `tauri-plugin-compmd` at 0.8.0 and `compmd-markdown` at 0.2.0, the wireframe pack at 0.1.0, the spec at 1.0-draft.16.

What to check:

- `165e9c2`: `@compmd/host` in `release.yml` (its version check, pack and stage) and `RELEASING.md`; every npm package's `repository.directory` after the move.
- `a7f892e`, `1d68a3f`: the spec cut as 1.0-draft.16; `SPEC_VERSION` and the READMEs naming it; the fork at `compmd-markdown` 0.2.0.
- `afdb48f`: the release notes in each package's README.
- `3cb590d`, `6174be5`: the versions, and the Tauri lockfiles outside the workspace.
- `7cf6e1b`: the renderer's highlight tests raced the queue on Linux (a `MessageChannel` message against the tests' `setTimeout(0)`); the queue's task is a timer in those tests, so their order is fixed.
- `89b8fb5`: the Tauri plugin's `include` anchored (`/src/**`, `/README.md`): unanchored, `README.md` matched every README under `node_modules`, and the crates' dry run refused the package. The tag `v0.8.0` was moved to this commit, with the maintainer's go, before anything was published.
- `53dc42a`: CMD-265, the wrong highlighting colours found by `cargo make linux`, left for later as decided.

The wireframe pack's release workflow is `4afcc8f`, before this range.

Left: deprecating the four 0.0.0 placeholders (`@compmd/blog`, `@compmd/create-blog`, `@compmd/host`, `@comp-markdown/wireframe`), by the maintainer.
