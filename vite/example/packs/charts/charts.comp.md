---
md:
  import:
    - from: ./dist/bar.js
      names: [bar]
  export:
    pack: "@example/charts"
    version: "1.0.0"
    names: [bar]
---

```compmd component=bar signature
params:
  values: array
```
