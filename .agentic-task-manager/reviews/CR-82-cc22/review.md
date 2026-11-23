+++
id = "01a20aa3-b424-7074-a628-60e283c9cc22"
number = 82
title = "CI on Linux: a made-up font URL, and a grid row in WebKit"
status = "approved"
tasks = ["CMD-393"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T23:00:30Z
updated = 2026-11-23T23:14:30Z
+++

Two checks the first push since 0.9.0 failed on CI's Linux runner, which the Mac passes (7a06d7e):

- **The link check:** a test of `crates/processor/src/resolve/load/fonts.rs` used `https://x.com/a.woff2` as a made-up address. It is a real site's, so `lychee` got 404. Now `https://a.org/a.woff2`, a placeholder host `lychee.toml` leaves out, as the sources' other made-up addresses use.
- **`follows a grid row that shrinks`** (`npm-packages/sandbox/test/sizing.test.ts`, WebKit) checked the shrunk row against 80 px. Its height is its text's: with Linux's fonts the page's own row is 100 px, and the framed box is the same, which the test's comparison of page and frame already checks. It now checks that the framed box shrinks, at the same width.

After the push, CI (its `ci` job, and the link check after one re-run that 503s from github.com had failed), Tauri and Pages are green on `main`.
