+++
id = "01a20ade-b36b-70a5-bd61-31f206c31e0c"
number = 83
title = "cargo make approve-release"
status = "open"
tasks = ["CMD-394"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-24T00:04:56Z
updated = 2026-11-24T00:04:56Z
+++

`cargo make approve-release`: the staged `@compmd/` packages of a release, approved as `cargo make approve-packs` approves the packs, as asked during 0.10.0, to use from now on.

- **`tools/packs/approve-staged.ts`:** `approve-packs`' flow, made one for a scope. Checkboxes, every version waiting chosen; the chosen ones approved in the order they were staged, with one one-time code; `--dry-run` lists them. It has `terminalSteps` and `runApproving`, which both commands share.
- **`approve-packs.ts`:** now its scope (`@comp-markdown`, "the packs") and nothing else; it does what it did.
- **`approve-release.ts`:** the same for `@compmd`, "the release".
- **`staged.ts`:** `stagedIn(json, scope)`; `stagedPacks` is it for the packs.
- **`Makefile.toml`** has the task, and **`RELEASING.md`**'s approval step names it; the by-hand way stays, with `npm stage list --json`.

## Tests
`approve-packs.test.ts` moved to `approve-staged.test.ts`, its tests unchanged, through `PACKS`. Two more for the release: its packages approved oldest first and no pack among them, and nothing staged said so. `cargo make scripts` passes (58). `cargo make approve-release --dry-run` against npm says nothing is staged: 0.10.0's packages are approved and published.
