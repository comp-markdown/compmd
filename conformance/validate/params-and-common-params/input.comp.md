```compmd component=card
params:
  title: string
  tone: info | warn = info
  count: number?
slots: [default, footer]
---
**{{title}}** {{md.slot}} {{md.slot.footer}}
```

::: card {title="Setup" tone=warn count=3 .wide #setup lang=de dir=auto width=40% md:padding="s l" order=first sticky=top show=md aspect-ratio=16/9 scroll}
Run it.

:: #footer
Two minutes.
::: /card
