---
md:
  import:
    - from: ./dist/counter.js
      names: [counter]
  export:
    pack: "@you/counter"
    version: "0.1.0"
    names: [counter]
---

# @you/counter

`counter`, a leaf in React, which counts the reader's clicks: `:: counter[Cups of tea: 0] {label="Cups of tea"}`.

```compmd component=counter signature
params:
  label: string
  start: number = 0
```
