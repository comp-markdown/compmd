+++
id = "01a1f9c3-335d-70a7-804a-1577c4ebc8c3"
number = 28
title = "The first packs: one per component, and WebAssembly in the frames again"
status = "open"
tasks = ["CMD-11"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T16:21:21Z
updated = 2026-11-20T19:14:32Z
+++

Round 3: releasing many packs (c5, c11), and the column examples (c8).

- **[Releases](docs/releases.md), decision 8, rewritten:** `cargo make pack-placeholders` (the `0.0.0` placeholder and its trusted publisher, by `npm trust github`), `compmd-packs/versions.json` with `cargo make release-packs` and one run of `release-packs.yml` per batch, and `cargo make approve-packs` (checkboxes, all chosen). Each with `--dry-run`. Mermaid and wireframe move to it.
- **Decision 10** here points to it; *What the review changed* lists c5 and c11.
- **Tasks:** CMD-350 (placeholders), CMD-351 (versions, release, workflow), CMD-352 (approving), in Packs 1.
- **`columns`:** `split` gains `wide-middle`, for three columns.
