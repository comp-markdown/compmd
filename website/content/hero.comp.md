---
title: Launch week
md:
  data: [{from: ./tasks.yaml, as: tasks}]
  inputs: {open: boolean = false}
---

# {{md.doc.title}}

::: grid {min=9rem gap=s}
::: box {padding=m background=subtle border-radius=m}
**:value[2]{of=(string(size(md.data.tasks.filter(t, t.done))))} of 3** done
::: /box
::: box {padding=m background=accent border-radius=m}
**:value[13]{of=(md.number(md.sum(md.data.tasks.filter(t, t.done).map(t, t.points)), 0))}** points shipped
::: /box
::: /grid

:: toggle-input[Only what's open] {bind=open}

::: each {in=(md.data.tasks.filter(t, !md.input.open || !t.done)) as=t join=list}
- {{t.name}}
::: /each
