---
md:
  version: 1
items: [a, b]
---

```compmd component=tag
params:
  label: string
---
Tag {{label}}.
```

Intro.

::: box
::: each {in=(md.doc.items) as=i}
:: tag[{{i}}] {label=(i)}
::: /each
::: /box
