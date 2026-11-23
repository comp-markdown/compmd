+++
id = "01a1ff10-d914-74ed-9166-bf9e0af14ad1"
number = 56
title = "The columns pack: two or three columns side by side"
status = "approved"
tasks = ["CMD-348"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T17:04:16Z
updated = 2026-11-21T17:10:06Z
+++

The columns pack, `@comp-markdown/columns` (item 4b of [the first components](docs/first-components.md)): a `compmd` definition on a grid, as the design sketched it ([no-code components](docs/no-code-components.md), decision 5, `md.slots.middle` choosing two tracks or three).

## What to check
- **Two changes from the task's text:**
  - **Side by side from `sm` up, not `md`.** A breakpoint follows the columns' box, not the screen, and an article's column is rarely 768 px (the test page's is 48rem less its padding), so `md:` columns would almost never be side by side.
  - **A `gap` param** (`space`, `m`): `gap` is `box`'s own param, not a common one, so a use couldn't change it otherwise (F008).
- `index.comp.md`: `split` (`even`, `wide-start`, `wide-middle`, `wide-end`), the tracks expression, the default, `middle` and `end` slots, the middle box only when given.
- `demo.comp.md`, each example a `#` section: two columns, three, a wide end (a recipe), a wide middle, right to left.
- **Right to left:** the demo sets `{lang=ar dir=rtl}` on the columns, since a document's or an example's `md.dir` reaches no renderer today: the expanded tree's root carries no `lang` or `dir`. Filed as **CMD-371**.
- Tests: `test/columns.test.ts`, the grid and its params, the column boxes, every `split` with two and three, F009 for an unknown split, `gap`, the plain reader in order, the demo; `test/page.test.ts` in Chrome and WebKit, three side by side in order, a wide column twice the other, the first on the right in right-to-left text, stacked in order on a phone.
- The README, `versions.json` and the workspace entry.
