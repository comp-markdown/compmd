+++
id = "01a1feb8-d4b9-772b-85a6-9850d4116c0b"
number = 50
title = "The link-card pack: a card that is a link"
status = "approved"
tasks = ["CMD-314"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T15:28:08Z
updated = 2026-11-21T15:37:56Z
+++

The link-card pack, `@comp-markdown/link-card` (item 4 of [the first components](docs/first-components.md)): a `compmd` definition on `card`, whose box has `stretch-link`, so a click anywhere on it follows its title's link ([no-code components](docs/no-code-components.md), decisions 1 and 3).

## What to check
- **One change from the design's sketch:** the title is the use's fallback (`md.fallback`, decision 2), as the figure's caption is, not a `title` param: it is written once, and is the link's text. `:: link-card[The harbour at dusk] {href="harbour.comp.md" description="…"}`.
- `index.comp.md`: `href: string` (the caller's, decision 1), `description: string?`, left out when not given; the title a heading one level below the one above the card.
- `demo.comp.md`: each use as a `{render=both}` example (a card, no description, the box changed by the use, three in a grid under "Read next").
- `test/link-card.test.ts`: the box with `stretch-link` and the use's params, the card in it, the heading and its link, the destination the document's, no description, the plain reader, the demo.
- The README, `versions.json` and the workspace entry. The click itself is the renderer's `stretch-link`, tested in Chrome and WebKit in CMD-361.
