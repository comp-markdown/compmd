---
md:
  import:
    - from: ./one.comp.md
      fonts: "*"
    - from: ./two.comp.md
      fonts: "*"
    - from: ./one.comp.md
      fonts: [missing, { name: serif, as: mono }]
    - from: ./gone.comp.md
      fonts: [lost]
---

::: box {font-family=sans}
Both documents give it.
::: /box

::: box {font-family=serif}
Only one gives it.
::: /box

::: box {font-family=lost}
From an import that can't be loaded.
::: /box
