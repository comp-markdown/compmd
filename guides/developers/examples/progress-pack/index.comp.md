---
md:
  import:
    - from: ./dist/progress.js
      names: [progress]
  export:
    pack: "@you/progress"
    version: "0.1.0"
    names: [progress]
---

# @you/progress

`progress`, a leaf: `:: progress[3 of 4 done] {value=3 max=4}`.

```compmd component=progress signature
params:
  value: number
  max: number = 100
```
