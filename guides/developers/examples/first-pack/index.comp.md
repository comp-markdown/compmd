---
md:
  import:
    - from: ./dist/hello.js
      names: [hello]
  export:
    pack: "@you/hello"
    version: "0.1.0"
    names: [hello]
---

# @you/hello

`hello`, a leaf: `:: hello[What a reader without it sees] {value="Hello"}`.

It is 12rem wide unless a use says otherwise (`width`, `height`, `aspect-ratio`), and `object-fit` and `object-position` say how it sits in a box of another shape.

```compmd component=hello signature {width=12rem max-width=100%}
params:
  value: string
```
