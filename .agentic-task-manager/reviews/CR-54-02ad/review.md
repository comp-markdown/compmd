+++
id = "01a1fef6-07ec-70f6-b314-5a4d04c302ad"
number = 54
title = "The design: type in a box"
status = "approved"
tasks = ["CMD-370"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T16:34:59Z
updated = 2026-11-21T16:58:50Z
+++

The design for type in a box (CMD-370, from CR-51), before any spec text or code: `docs/type-params.md`.

## To settle
- **Decision 1, which params:** `font-size` (`s`, `m`, `l`, `xl` in `em`, or a length), `font-weight` (`normal`, `bold`), `font-style` (`normal`, `italic`), `font-family` (tokens only: `body`, `heading`, `mono`), `color` (`muted`, `accent`, or a CSS colour), `text-align` (`start`, `center`, `end`), all with breakpoints. Rejected: line height, letter spacing, transform, decoration; font names; numeric weights.
- **Decision 2:** the tokens as custom properties with defaults, a host's theme changing them.
- **Decision 3:** inherited, as CSS's are.
- **Decision 4:** W126 for a `color` and a `background` both CSS colours on one box, below 4.5:1; nothing for tokens.
- **Decision 5 and G1, open:** a host-language component's box applies them, so what draws in the page inherits them; should a sandboxed component get them in its context, as it gets `fit` and `scheme`?
- **The examples:** a project's badges, a pull quote, a muted footnote, a masthead, a card's title.
