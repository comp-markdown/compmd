---
title: Release
md:
  inputs:
    region: all | eu = all
---

```compmd component=card
params:
  card-title: string
slots: [default, footer]
---
**{{card_title}}** in {{md.doc.title}}: {{md.slot}} {{md.slot.footer}} {{md.slot.aside}} {{md.input.region}} {{title}}
```

::: card {card-title="A {{md.doc.title}} {{md.doc.nope}}"}
Text.
::: /card
