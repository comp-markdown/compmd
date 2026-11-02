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

`__component__` draws every ` ```__language__ ` block of a document that imports it, in the page's colour scheme; a reader without it sees the code. Written as a leaf, its `code` is a param, for source made from data: `:: __component__[What it shows] {code=("…")}`.

```compmd component=__component__ signature
renders: __language__
```
