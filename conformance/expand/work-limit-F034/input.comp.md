---
l: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
---

Loops that make nothing still count their work.

::: each {in=(md.doc.l) as=a}
::: each {in=(md.doc.l) as=b}
::: each {in=(md.doc.l) as=c}
::: each {in=(md.doc.l) as=d}
::: /each
::: /each
::: /each
::: /each

After the loops.
