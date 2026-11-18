---
md:
  import:
    - from: ./dist/__component__.js
      names: [__component__]
  export:
    pack: "__pack__"
    version: "0.1.0"
    names: [__component__]
---

# __pack__

`__component__`, a leaf: `:: __component__[What a reader without it sees] {value="Hello"}`.

```compmd component=__component__ signature
params:
  value: string
```
