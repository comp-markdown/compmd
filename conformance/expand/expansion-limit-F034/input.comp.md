---
md:
  data:
    - from: ./rows.yaml
      as: rows
    - from: ./cols.yaml
      as: cols
---

::: each {in=(md.data.rows) as=r}
::: each {in=(md.data.cols) as=c}
{{r}}.{{c}}
::: /each
::: /each
