+++
id = "01a1f8ba-be48-7059-889f-0140db244ac5"
number = 26
title = "Release 0.9.0"
status = "approved"
tasks = ["CMD-345"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T11:32:30Z
updated = 2026-11-20T11:38:58Z
+++

Release 0.9.0, as [releases](docs/releases.md) says: every npm package and the crates at one version, a minor one since it breaks.

What to check:

- **The spec's revision:** `1.0-draft.17` (c4834ba, tagged `spec-v1.0-draft.17`), for CMD-308's §8.6 and §15 change; `SPEC_VERSION` in the crate and `@compmd/processor`, and the implementing packages' READMEs, name it.
- **The release notes** (c569105), each package's 0.9.0 line: the host's highlighter and protocol version 2, breaking (CMD-262); every origin for a host's own documents and the blog's share image (CMD-308); only the host components a document uses (CMD-268); the blog's `sharedFiles`, its exported helpers and trusted packs' Svelte components (CMD-13). The packages that didn't change say so.
- **The version** in every `package.json`, the crates' `Cargo.toml` and `Cargo.lock` (8a7cb67), and the Tauri lockfiles (87032ac).
- **The fork** (`compmd-markdown`) hasn't changed since 0.8.0: not published again.
- **The tag** `v0.9.0` is made locally, on 87032ac.

Checks: `cargo make ci` green (one run lost the observer's O6 test in Chromium to a timeout under load; it passed alone and on the rerun), `cargo make tauri-check` on macOS green, `cargo make site` green for CMD-262.

After approval, with the user's go: `git push --follow-tags`, which pushes `main` with `spec-v1.0-draft.17` and `v0.9.0`; the release workflow stages the npm packages for the user's approval and waits for the crates' in `crates-io`.
