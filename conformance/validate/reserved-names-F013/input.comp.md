---
items: [a]
md:
  inputs:
    type: string = ""
---

```compmd component=chip
params:
  type: string
---
Chip.
```

::: each {in=(md.doc.items) as=int}
Item.
::: /each
