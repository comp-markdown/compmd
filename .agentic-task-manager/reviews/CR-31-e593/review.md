+++
id = "01a1fac4-66b4-708b-b6dd-01a538fde593"
number = 31
title = "Packs: one file of versions, one command and one workflow"
status = "open"
tasks = ["CMD-351"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T21:02:17Z
updated = 2026-11-20T21:02:17Z
+++

One file of versions, `cargo make release-packs` and one workflow, `release-packs.yml` ([releases](docs/releases.md), decision 8). `cargo make ci` is green.

What to check:

- **`compmd-packs/versions.json`** (Mermaid 0.1.2, wireframe 0.1.0) and **`cargo make pack-versions`**, now in `ci`: a pack it doesn't name, or a `package.json` or `export.version` that differs, fails.
- **`cargo make release-packs`**, with `--dry-run`: writes the versions into the packs' files when they differ, then stops for a commit; lists the packs npm lacks; refuses unless `main`'s commit is pushed and the packs committed, and a tag at another commit; asks, then tags `<pack>-v<version>`, pushes the tags and runs the workflow once (`gh workflow run`). Dry run against the real registry: nothing to release.
- **The workflow is dispatched, not triggered by the tags:** GitHub starts nothing for a push of more than three tags at once. It stages only packs **tagged at its commit** whose version npm lacks (`tools/packs/stage-packs.ts`), the guard the old workflows' "tag is the version" step gave. It runs `cargo make ci` once.
- **`release-mermaid.yml` and `release-wireframe.yml` are removed.** The two packs' trusted publishers still name them, so their next release needs the publisher moved first: RELEASING.md, *The packs*, has the three `npm trust` commands (list, revoke, github), checked against the pinned npm's help. That step is yours, with your 2FA.
- **Not run for real:** the workflow needs to be on `main`, and a release needs a version to release.
