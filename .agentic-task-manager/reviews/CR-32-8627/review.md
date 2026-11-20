+++
id = "01a1fad0-f7fd-7159-8284-c1441f068627"
number = 32
title = "Packs: approve what is staged, chosen from a list"
status = "open"
tasks = ["CMD-352"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T21:16:01Z
updated = 2026-11-20T21:16:01Z
+++

`cargo make approve-packs`, with `--dry-run` ([releases](docs/releases.md), decision 8). `cargo make ci` is green.

What to check:

- **The list:** `npm stage list --json`, the packs' items only (`@comp-markdown/`), oldest first, so a pack is never on npm before one staged before it. Checkboxes with `@inquirer/checkbox` 5.2.8 (MIT; 5.3.0 is ten days old), every version waiting chosen; Enter approves the chosen ones in staging order, whatever order the list returns them in.
- **The fields** (`id`, `packageName`, `version`, `status`, `createdAt`) are from npm 11.21.1's own source (`lib/utils/key-values.js`), since its docs don't give them. **`status`'s values aren't documented anywhere:** an item starts unchosen only when its status says approved, rejected or published. If npm's list holds only items waiting, that never matters.
- **The code:** asked once (Enter to leave it to npm, for a security key) and passed as `--otp` to each approval; npm's `otplease` asks for a new one on the terminal when it has expired and retries (checked in its source).
- **`tools/packs` is a workspace member now,** for that one dependency; no scripts, so the workspace's runs skip it, as the spikes.
- **npm's failures** in all three tools are one line now, with npm's own summary, and `npm login` named on E401; found by mistake: I ran `approve-packs` without `--dry-run` while checking it loaded, and npm answered E401 (your token on this machine isn't valid any more). It only read, and failed; nothing changed.
- **Not run for real:** it needs your npm login and something staged.
