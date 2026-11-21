---
md:
  export:
    pack: "@comp-markdown/timeline"
    version: "0.1.0"
    names: [timeline, event]
---

# @comp-markdown/timeline

`timeline` is dates and what happened, along a line: a harbour's history in a post, a project's milestones in notes, a trip day by day. Each `event` in it has a dot on the line, its date small and muted, a title, and what you write in it.

```md
::: timeline
::: event {date=1897 title="The harbour wall"}
Built in a single summer, of granite from the quarry.
::: /event

::: event {date=2024 title="The lamps come back" tone=accent}
All forty of them, restored.
::: /event
::: /timeline
```

- **`date`** is written as you write it: `1897`, `May 3`, `Day 2`.
- **`title`**, which an event may leave out, is in bold under the date.
- **`tone=accent`** fills the event's dot with the accent colour, for the current or most important one; `subtle`, the default, is a muted dot.
- **What you write in an event** is under its title: paragraphs, links, a list.

A quick timeline is a list, with no events: `::: timeline` around `- **1897**: the harbour wall is built.`, the list's markers on the line. The line is the timeline's start border, `{border-inline-start=subtle}` for a fainter one. It prints as it shows. In a plain Markdown reader, a timeline is its events' text, each date, title and text in order.

```compmd component=timeline {border-inline-start=strong padding-inline-start=m}
slots: [default]
---
{{md.slot}}
```

```compmd component=event {padding-block-end=m}
params:
  date:
    type: string
    doc: When it happened, as you write it
  title:
    type: string?
    doc: What happened, in bold under the date
  tone:
    type: subtle | accent
    default: subtle
    doc: The dot's colour, accent for the current or most important event
slots: [default]
---
::: box {inset-inline-start=-1.375rem inset-block-start=0.4rem width=0.75rem height=0.75rem border-radius=full background=currentColor color=(tone == "accent" ? "accent" : "muted")}
::: /box

::: box {color=muted font-size=s}
{{date}}
::: /box

::: if {when=(title != null)}
**{{title}}**
::: /if

{{md.slot}}
```
