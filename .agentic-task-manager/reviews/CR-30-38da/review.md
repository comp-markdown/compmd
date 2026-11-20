+++
id = "01a1fab2-1ffd-7112-88c8-332e635238da"
number = 30
title = "Packs: placeholders and trusted publishers in one command"
status = "open"
tasks = ["CMD-350"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T20:42:20Z
updated = 2026-11-20T20:42:20Z
+++

`cargo make pack-placeholders <pack>...`, with `--dry-run` ([releases](docs/releases.md), decision 8).

What to check:

- **`tools/packs/`, one concept a module:** `registry.ts` asks the registry over HTTP (npm doesn't run inside the repository); `npm.ts` runs npm 11.21.1, the release workflows' pin, from a temporary folder, in the terminal, so npm asks for your one-time code itself; `placeholder.ts` writes the placeholder; `pack-placeholders.ts` runs it.
- **The placeholder:** `0.0.0` with no code, MIT (`LICENSE-CODE`), and `repository` naming `compmd-packs/<pack>`, which npm's provenance checks against the trusted publisher.
- **The trusted publisher:** `npm trust github <package> --file release-packs.yml --repository comp-markdown/compmd --allow-stage-publish --yes`, checked against the pinned npm's own help. `release-packs.yml` comes with CMD-351.
- **A 2-second pause** between npm's calls, and a reminder at the end that a trusted publisher expires after 2 days without a publish.
- **Not run for real:** it would publish to npm with your account. `--dry-run` against the real registry: `card` and `embed-youtube` would be created, Mermaid left as it is.
- **RELEASING.md**, *A new pack*; the tests in `cargo make scripts`, which `ci` runs.
