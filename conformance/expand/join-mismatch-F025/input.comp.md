---
items: [a, b]
---

::: each {in=(md.doc.items) as=i join=list}
Not a list item: {{i}}
::: /each
