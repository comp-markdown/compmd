+++
id = "01a20b23-5c5f-7034-8b15-e186a13f3818"
number = 84
title = "Release 0.10.0, the first packs and the post"
status = "approved"
tasks = ["CMD-390"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-24T01:19:56Z
updated = 2026-11-24T01:20:21Z
+++

CompMD 0.10.0, the first packs and the post *The first packs*: released and published (2026-11-23).

## What was done
- **The review records:** CR-31 to CR-80, never committed before (b1f5665).
- **The spec's revision `1.0-draft.18`**, tagged `spec-v1.0-draft.18`. `SPEC_VERSION` and the READMEs name it (d15cb4c).
- **Release notes**, a 0.10.0 entry in 14 package READMEs (d15cb4c). They were drafted by reading every task since 0.9.0, then checked: the sandbox's protocol is version 3, and breaking notes were added for the Rust crate's public struct fields and for a host's CSS against the renderer.
- **The version 0.10.0** in every package and crate (62f593b), and the Tauri lockfiles (08abf01).
- **Pushed main** with the spec's tag. CI's native Linux runner found two failures the Mac and the emulated container didn't, fixed in CMD-393 (CR-82), and a flaky observer test, fixed in CMD-392 (CR-81).
- **`v0.10.0`** tagged at 7a06d7e, the commit CI passed. The release workflow's first run timed out in a create-pack test (5.07 s against 5 s); its re-run passed. The 11 npm packages were approved, and the 4 crates published.
- **The packs:**
  - 9 placeholders, by the maintainer.
  - `release-packs`: its first run staged 6 and failed at Mermaid with `ENEEDAUTH`, because Mermaid's trusted publisher was still `release-mermaid.yml`'s. The maintainer moved it to `release-packs.yml` and ran it again for the other 4.
  - All 10 approved: mermaid 0.1.3, gallery and sticky-note 0.2.0, the rest 0.1.0.
- **The post** out of draft (2d0f1c2), checked on compmd.dev: the badges' gradient, the sticky note's cut corner, the rating's stars, and the post in the feed.

## Found along the way
- CMD-391 (backlog): a font in a pack fetched from a registry while a document is open probably doesn't load.
- CMD-394 (done): `cargo make approve-release`, to approve a release's staged packages from now on.
- The rating's frames take a few seconds on a cold load, and the spot is blank meanwhile, not its fallback. Not a task yet.
- The wireframe pack's trusted publisher is still its old workflow's. Move it when its next version goes out.
