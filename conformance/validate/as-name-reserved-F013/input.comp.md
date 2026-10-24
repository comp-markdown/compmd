---
items: [1, 2]
---

::: each {in=(md.doc.items) as=each}
{{each}}
::: /each

::: each {in=(md.doc.items) as=md}
Item.
::: /each
