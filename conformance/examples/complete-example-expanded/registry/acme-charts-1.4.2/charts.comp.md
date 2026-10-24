---
md:
  import:
    - from: ./chart.jsx
      names: [chart]
  export:
    pack: "@acme/charts"
    version: "1.4.2"
    names: [chart]
---

```compmd component=chart signature
params:
  rows: array
  type: bar | line = bar
  selected: input<string>?
```
