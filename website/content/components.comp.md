```compmd component=card {padding=m border=subtle border-radius=m}
params:
  title: string
slots: [default, footer]
---
**{{title}}**

{{md.slot}}

{{md.slot.footer}}
```

::: card {title="Ship the beta"}
Three small tasks left.

:: #footer
*Due Friday.*
::: /card
