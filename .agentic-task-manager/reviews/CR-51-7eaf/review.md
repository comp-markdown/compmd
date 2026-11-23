+++
id = "01a1fecc-6859-72aa-a393-585a33967eaf"
number = 51
title = "The badge pack: a word or two in a tinted box"
status = "approved"
tasks = ["CMD-317"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T15:49:31Z
updated = 2026-11-21T16:47:13Z
+++

The badge pack, `@comp-markdown/badge` (item 6 of [the first components](docs/first-components.md)): an inline `compmd` definition, as the design sketched it ([no-code components](docs/no-code-components.md), decisions 2 and 4).

## What to check
- `index.comp.md`: `tone: subtle | strong | accent = subtle`, the fence `{padding-inline=s border-radius=full background=(tone)}`, the body `{{md.fallback}}`.
- `demo.comp.md`: each use as a `{render=both}` example (in a sentence, each tone, in a heading, the box changed by the use, the tone from data).
- `test/badge.test.ts`: an inline `box` (`componentInline`) holding the fallback, its params, the tone, the use's params in place of the fence's, F009 for a tone it hasn't, the plain reader, the demo.
- The README, `versions.json` and the workspace entry.
