+++
id = "01a20941-4210-7609-b4d0-34b734edcb11"
number = 76
title = "sticky-note 0.2.0: yellow, a pin and a turned-up corner"
status = "approved"
tasks = ["CMD-386"]
author = "claude:a7b49598"
reviewers = []
created = 2026-11-23T16:33:21Z
updated = 2026-11-23T21:13:03Z
+++

`@comp-markdown/sticky-note` 0.2.0: a note that reads as a note on any page, as settled with the maintainer on 2026-11-23.

## Since round 1: the corner, cut (c1)
The yellow left where the corner turns up is gone. With gradients in `background` (spec §9.4, CMD-387, CR-78; the code in CMD-388, CR-80), the note cuts its own corner, and the page shows through:
- **The note:** `background="linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)"`. The angle is logical, so the cut is at the bottom end corner, at 45° on any note, and at the bottom left in right-to-left text.
- **The flap:** a 1.98rem square at the bottom end with `linear-gradient(315deg, transparent 50%, #e0c93c 50%)`, the triangle the cut leaves out (1.4rem × √2), turned over. It replaces the two borders.
- **Still there:** the note's shadow follows its rectangle, so a thin shadow shows past the cut, a gap the design lists.
- **A use that changes `background`** paints the whole note again, corner included; the flap stays.

Checked in the post, light and dark. `cargo make ci` passes.

## What 0.2.0 changes
- **Classic yellow with dark text by default** (`#fff176`, `#3b3000`), opaque, in place of the page's `accent` tint, which a theme may make see-through.
- **A red pin at its top:** a round box at `inset-inline-start=46%` (no `calc()` for an inset), with `padding-block-start=l` to leave room.
- **The pin and the flap have no text**, so a screen reader reads the note alone, and a plain Markdown reader shows its text.
- **Version 0.2.0** in `versions.json`, `package.json` and the export; README, demo and designs updated. A use that changes the background sets a text colour too: `{background=subtle color=accent}`.

## Tests
`sticky-note.test.ts` (the fence, the pin and the flap as placed boxes with no text) and `page.test.ts` in Chrome and WebKit (the note's gradient, the pin, the flap at the bottom end).

## Before the post that uses it is published
0.2.0 has to be released to npm (`cargo make release-packs`), after CMD-388's processor and renderer, which it needs.
