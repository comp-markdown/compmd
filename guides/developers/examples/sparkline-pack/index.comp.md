---
md:
  import:
    - from: ./dist/sparkline.js
      names: [sparkline]
  export:
    pack: "@you/sparkline"
    version: "0.1.0"
    names: [sparkline]
---

# @you/sparkline

`sparkline`, a leaf: `:: sparkline[Visits rose all week] {values=[3, 5, 4, 8, 12]}`.

```compmd component=sparkline signature
params:
  values: array
```
