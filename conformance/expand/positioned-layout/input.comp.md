---
notes:
  - {text: "Skipped Tuesday", x: "10%"}
  - {text: "Best streak", x: "55%"}
---

```compmd component=board {height=6rem border=subtle}
slots: [default]
---
{{md.slot}}
```

::: box {height=12rem padding=m}
This week's notes, pinned where they matter.

::: box {inset-inline-start=60% inset-block-start=3rem width=30%}
A note placed at 60% across and 3rem down.
::: /box

::: box {.badge inset-inline-end=8px inset-block-start=8px}
3 new
::: /box

::: box {inset-inline-start=0 inset-inline-end=0 inset-block-end=0 md:inset-inline-start=auto md:inset-inline-end=auto md:inset-block-end=auto}
A bar along the bottom on a phone; in the flow from md up.
::: /box
::: /box

::: box {height=4rem}
::: each {in=(md.doc.notes) as=n}
::: box {inset-inline-start=(n.x) inset-block-start=0}
{{n.text}}
::: /box
::: /each
::: /box

::: board
::: stack {.pin inset-inline-start=1rem inset-block-start=1rem}
Placed in the board's box, through its slot.
::: /stack
::: /board
