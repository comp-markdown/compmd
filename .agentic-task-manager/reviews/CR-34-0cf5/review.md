+++
id = "01a1faf9-5838-7685-8858-992b3f900cf5"
number = 34
title = "The card: a box with an image slot, its text and a footer"
status = "approved"
tasks = ["CMD-313"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T22:00:07Z
updated = 2026-11-20T22:11:49Z
+++

`@comp-markdown/card`, the first no-code pack: a `box` with an image on top, then what the author writes, then a footer.

## What to check

- **The image is a slot, not a param** (`:: #image`). This is a decision for you. As a slot, the image is the document's own Markdown image: its path resolves against the document, its alt text is written where it's used, and a plain Markdown reader shows it. An `image=` param would have to be a URL string, with the alt text as a second param, and a plain reader would show nothing. The cost is one more line per card. The title is the heading the author writes, at the level the document needs, so it's not a param either.
- **The box's defaults** come from the signature fence's param list (`{padding=m border=subtle border-radius=m}`), which spec §7.4 now gives to the box (CMD-353, in CR-33). A use's params go after them and win: `::: card {background=subtle border=none}`. This pack depends on CR-33's spec text.
- **Order:** in the expanded tree, image, text, footer. In a plain Markdown reader the slots come where the author wrote them (text, then image, then footer), and the test checks that.
- `demo.comp.md`: three cards in a `grid`, with two local SVG pictures (`harbour.svg`, `orchard.svg`), so the demo needs nothing from the network.
- `versions.json` adds `"card": "0.1.0"`; `cargo make pack-versions` agrees. The placeholder and its trusted publisher are still to do (`cargo make pack-placeholders card`, your login).

## Verified

`cargo make ci`: everything passed, except `website/build/snapshots.test.ts`, whose setup hook took longer than 10 s under ci's load. It passes alone (`pnpm --dir website test`), and is now CMD-358. `scripts` and `pack-versions`, which ci didn't reach, pass when run alone. The card's own 5 tests pass in ci. The test page (`compmd-pack try`) has no diagnostics in light or dark.
