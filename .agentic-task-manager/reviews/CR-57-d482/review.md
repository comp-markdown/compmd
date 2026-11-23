+++
id = "01a1ff5b-35a0-759a-8a0b-f819ae98d482"
number = 57
title = "Type in a box: ten params, their tokens, W126"
status = "approved"
tasks = ["CMD-370"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T18:25:30Z
updated = 2026-11-21T19:57:01Z
+++

CMD-370 built from [its approved design](docs/type-params.md) (CR-54), decisions 1 to 4 and 6: type in a box. Decision 5 (a component's type) is CMD-372, decision 7 (fonts of a document's own) CMD-373.

## What to check
- **The spec:** §9.2's table (the ten params) and a *Type* paragraph (inherited, sizes in `em`, `font-family` tokens only, `text-transform` showing not changing the text, print, plain renderers), a *Contrast* paragraph (W126); §9.4's type tokens and their custom properties; §10.2's W126; the changelog, breaking for a signature that declares one of the names (`color`); `llms.txt`.
- **One narrowing of the design:** W126 checks colors written in hex, `rgb()` or `hsl()`, opaque, at each breakpoint. A named color, other functions and tokens aren't checked: the processor has no RGB for them.
- **The processor:** `validate/universal.rs` (the params, their kinds, breakpoints), `values.rs` (`FONT_SIZE`, `TEXT_COLOR`, `LINE_HEIGHT`, `LETTER_SPACING`), `expand/contrast.rs` (WCAG 2's ratio, with unit tests), W126 in `expand/layout.rs`, `params.json`.
- **Conformance:** `validate/type-params` (every param with tokens, lengths, a color, a number, breakpoints, an inline box; F009 for a font name, a numeric weight, a physical side; W126 for a light grey on white and from `md` up; none for a dark grey or tokens). `validate/host-signature-params-F008-F009` used `color` as its unknown param; it is `tint` now.
- **The renderer:** `styles.ts` (the tokens and the CSS); `test/browser/type.test.ts` in Chrome and WebKit (a badge's size in a heading and in text, inheritance into nested boxes until one sets its own, a theme changing a token, a transform showing capitals with the text as written).
- **The guides:** the Author's Guide's chapters 3 (the table, a paragraph), 19 and 20 (W126); the Developer's Guide's chapter 22; its rating example's `color` param renamed `tint` (chapter 14 and the example pack).
- **The badge:** `labels.comp.md` with colours and type (`beta` in spaced capitals), its demo and README.
- **Not here: a print test**, as for CMD-366: the rules have no media query; the renderer's browser tests can't emulate print.
