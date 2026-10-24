```compmd component=callout
params:
  kind: tip | warning = tip
slots: [default]
---
::: box {pad=1rem radius=8px border="1px solid" class=("callout-" + kind)}
{{md.slot}}
::: /box
```
