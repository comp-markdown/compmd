---
md:
  export:
    names: [notice]
---

A box with a title, for what a reader should see first.

```compmd component=notice {padding=m border-radius=m background=subtle}
params:
  title: string
slots: [default]
---
**{{title}}**

{{md.slot}}
```
