---
md:
  export:
    pack: "@comp-markdown/timeline"
    version: "0.1.0"
    names: [timeline]
---

# @comp-markdown/timeline

`timeline` is dates and what happened, along a line: a harbour's history in a post, a project's milestones in notes, a trip day by day.

```md
::: timeline
- **1897**: the harbour wall is built.
- **1952**: the quay is rebuilt after the storm.
- **2024**: the lamps along the quay come back.
::: /timeline
```

What you write in it is the timeline: a list, each item a date and what happened, the date as you write it (`**1897**`, `**May 3**`, `**Day 2**`). A line runs down its start side, and the list's own markers mark each item. A use changes its box as any box's: `{border-inline-start=subtle}` for a fainter line, `{padding-inline-start=l}` for more room. It prints as it shows. In a plain Markdown reader, a timeline is its list.

```compmd component=timeline {border-inline-start=strong padding-inline-start=m}
slots: [default]
---
{{md.slot}}
```
