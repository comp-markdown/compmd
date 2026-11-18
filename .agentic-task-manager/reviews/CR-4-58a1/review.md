+++
id = "01a1ec4d-c700-7679-8832-ad6bdff158a1"
number = 4
title = "The repository's top-level folders, organized"
status = "approved"
tasks = ["CMD-246"]
author = "claude:79612601"
reviewers = []
created = 2026-11-18T01:38:02Z
updated = 2026-11-18T03:15:24Z
+++

The repository's folders grouped by what they hold. Moves and path updates only: nothing behaves differently.

## The layout

| Folder | From |
|---|---|
| `spec/` | `compmd-spec-v1.md`, `compmd-registry-v1.md`, `CHANGELOG.md`, `llms.txt`, `examples/complete` |
| `crates/` | `processor`, `host`, `pack-types`, `tauri`, the `markdown-rs` submodule |
| `crates/tauri/examples/tauri-app/` | `examples/tauri` |
| `npm-packages/` | `render`, `sandbox`, `devtools`, `registry`, `vite`, `pack`, `create-pack`, `blog`, `create-blog`, `editor`, `wasm`, `host-js` |
| `compmd-packs/` | `packs/mermaid`, `packs/wireframe` |
| `docs/existing-specs-comparison.md` | the root |
| `branding/` | `logo/` |
| `tools/linux-ci/`, `tools/spec-revision/` | `docker/`, `scripts/` |

Every folder moved with `git mv`, so `git log --follow` keeps its history.

## What to check

- **The README's layout section** replaces its table of files: a table for each top-level folder, and a line on `wasm/` and `host-js/` being a crate and a package both.
- **The configuration**: the Cargo workspace's `members` and `exclude`, `pnpm-workspace.yaml` and the lockfile's importers, `.gitmodules`, `Makefile.toml`, the workflows (their path filters, `--dir` and `--manifest-path`), `publish-crates.sh`, `lychee-inputs.txt`, `tools/linux-ci/run.sh`.
- **The site serves the same URLs**: `/spec/` from `spec/` (`SPEC_FOLDER` in `website/build/companion.ts`, so a heading's source link and the spec's relative links point to `spec/` on GitHub), `/logo/` from `branding/`.
- **Links**: relative links were rewritten from each file's old place to its target's new place, and the absolute GitHub links in the published READMEs and API docs to the new paths. Mentions of a file in prose (`` `processor/src/lib.rs` ``) name its new path. History stays as it was: the changelog's entries and a design's record of what changed.
- **Tasks**: the unfinished tasks that named an old path (CMD-1, 3, 4, 5, 6, 9, 11, 12, 17, 18, 19, 22, 23, 31) name the new one.

## The spec

The spec's text changes only in its links (its logo, the tree schema, the renderer's README), but `spec-check` compares `spec/compmd-spec-v1.md` with the revision's tag, where the file was at the root, so the spec has a revision of its own, 1.0-draft.15 (tag `spec-v1.0-draft.15`), whose changelog entry names the folder. `cargo make spec-check` passes.

## Checks

`cargo make ci`, `cargo make site`, `cargo make links` and `cargo make tauri-check` pass.
