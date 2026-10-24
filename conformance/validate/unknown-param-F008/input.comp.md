```compmd component=card
params:
  title: string
  tone: info | warn = info
  count: number?
slots: [default, footer]
---
**{{title}}** {{md.slot}} {{md.slot.footer}}
```

::: card {title="Setup" colour=red}
Run it.
::: /card
