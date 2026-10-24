```compmd component=card
params:
  title: string
  tone: info | warn = info
  count: number?
slots: [default, footer]
---
**{{title}}** {{md.slot}} {{md.slot.footer}}
```

::: card {tone=warn}
Run it.
::: /card
