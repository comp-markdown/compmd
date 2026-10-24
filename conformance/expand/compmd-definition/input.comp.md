```compmd component=card
params:
  title: string
  tone: info | warn = info
slots: [default, footer]
---
**{{title}}** ({{tone}})

{{md.slot}}

{{md.slot.footer}}
```

::: card {title="Setup"}
Run `rayplay init`.

:: #footer
Two minutes.
::: /card
