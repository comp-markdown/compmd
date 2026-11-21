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

It is 12rem wide unless a use says otherwise (`width`, `height`, `aspect-ratio`), and `object-fit` and `object-position` say how it sits in a box of another shape.

```compmd component=__component__ signature {width=12rem max-width=100%}
params:
  value: string
```
