---
tasks:
  - {title: "Write spec", status: done}
  - {title: "Build parser", status: doing}
---

```compmd component=board
params:
  rows: array
  by: string
  as: name
slots:
  default: {each: rows, as: as}
---
{{md.slot}}
```

::: board {rows=(md.doc.tasks) by=status as=t}
**{{t.title}}**: {{t.status}}, {{md.loop.number}}
::: /board
