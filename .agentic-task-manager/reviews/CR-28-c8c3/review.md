+++
id = "01a1f9c3-335d-70a7-804a-1577c4ebc8c3"
number = 28
title = "The first packs: one per component, and WebAssembly in the frames again"
status = "open"
tasks = ["CMD-11"]
author = "claude:79612601"
reviewers = []
created = 2026-11-20T16:21:21Z
updated = 2026-11-20T19:05:03Z
+++

Round 2: your two comments, agreed in conversation.

- **`columns` (row 4b):** a `compmd` definition on `grid`, the first column the default slot, `:: #middle` and `:: #end` the others, `split=even | wide-start | wide-end`, stacking below `md`. Decision 2 no longer counts columns as covered by `row` and `grid`. Its task: CMD-348, in Packs 1.
- **`word-cloud` (row 10b):** a leaf over data rows laid out by `d3-cloud`, drawn as SVG; its size and text measuring in a strict frame are checked first. Its task: CMD-349, in Packs 2, after CMD-9.
- **Charts lose `height`:** CMD-8's spike found the common params (`height`, `width`, …) are the box's and never reach a component, so the chart fills its box and the use sizes it (`height=20rem`). CMD-321's description says so.
- *What the review changed* lists CR-28's comments.
