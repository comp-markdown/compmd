+++
id = "01a1ff07-ae06-738b-942c-1293e6d53e06"
number = 55
title = "The timeline pack: dates and what happened, along a line"
status = "approved"
tasks = ["CMD-319"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T16:54:15Z
updated = 2026-11-21T19:59:49Z
+++

The timeline pack, `@comp-markdown/timeline` (item 8 of [the first components](docs/first-components.md)): a `compmd` definition, as the design sketched it ([no-code components](docs/no-code-components.md), decision 6): a box with a line down its start side (`border-inline-start=strong padding-inline-start=m`), holding the list the use writes, the dates in the caller's text.

## What to check
- `index.comp.md`: the fence and the default slot; the README and the page's text.
- `demo.comp.md`, with each example a `#` section (CR-51): years, days of a trip with paragraphs, a link and a nested list, the box changed by the use, and the items from `milestones.yaml` with `each` and `join=list` (without it, W103: each item a list of its own).
- `test/timeline.test.ts`: the box and its params, the list, the use's params in place of the fence's, the items from data in one list, the plain reader, the demo.
- The README, `versions.json` and the workspace entry.
