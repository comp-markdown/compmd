```compmd component=card
params:
  title: string
  tone: info | warn = info
  count: number?
slots: [default, footer]
---
**{{title}}** {{md.slot}} {{md.slot.footer}}
```

::: card {title="Setup" tone=loud count=three width=wide md:show=md xl:padding=s}
Run it.
::: /card
