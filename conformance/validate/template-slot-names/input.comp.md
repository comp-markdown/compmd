---
tasks: []
---

```compmd component=board
params:
  rows: array
  by: string
  as: name
slots:
  default: {each: rows, as: as}
  footer: {}
---
{{md.slot}}

{{md.slot.footer}}
```

::: board {rows=(md.doc.tasks) by=status as=t}
**{{t.title}}**, number {{md.loop.number}}

:: #footer
{{t.title}} isn't bound here.
::: /board
