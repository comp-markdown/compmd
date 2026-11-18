---
md:
  import:
    - from: ./dist/bar.js
      names: [bar]
    - from: ./Total.jsx
      names: [{name: Total, as: total}]
  export:
    pack: "@example/charts"
    version: "1.0.0"
    names: [bar, total]
---

```compmd component=bar signature
params:
  values: array
```

```compmd component=total signature
params:
  values: array
```
