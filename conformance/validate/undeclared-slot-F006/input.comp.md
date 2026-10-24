```compmd component=card
params:
  title: string
  tone: info | warn = info
  count: number?
slots: [default, footer]
---
**{{title}}** {{md.slot}} {{md.slot.footer}}
```

::: card {title="Setup"}
Run it.

:: #aside
Not a slot of card.
::: /card
