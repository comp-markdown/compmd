+++
id = "01a1f2d0-025a-7202-a7b6-388d074bea02"
number = 11
title = "Writing a block’s rules: measured, no change"
status = "approved"
tasks = ["~CMD-254"]
author = "claude:79612601"
reviewers = []
created = 2026-11-19T07:58:00Z
updated = 2026-11-19T08:14:56Z
+++

CMD-254 measured first, as it asked, and the numbers support no change: the renderer keeps writing the document's sheet whole.

## What to check

- **The measurement** (`spikes/highlight-perf/src/main.ts`, `churnSheet`; `measure.ts`'s `sheet-*` and `churn-styled-nowrite`): in the 20,000-element host, 100 blocks with a rule and a container-query rule each, written as the whole sheet, rule by rule, or not at all; and the renderer's styled pages with their sheet writes counted but not made.
- **The finding** (`spikes/highlight-perf/README.md`, *Writing a block's rules*): in WebKit an open is 16 ms with no rules written, 17–18 ms with the whole sheet, 17–19 ms rule by rule; the renderer's styled pages are no faster without their writes (20 ms either way). What a styled open costs beyond a code block's is its own boxes.
- **Not measured:** hundreds of styled pages live at once; that is what could change the answer.
- **Noted:** the CMD-253 run measured styled opens at 38 ms, this one at 20 ms on the same page; the difference wasn't looked into. Comparisons are within a run.

No package changes. The tests the task named (rule order, a destroyed page's rules leaving the sheet) are CMD-253's, in `styles.test.ts`.
