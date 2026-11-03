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

```compmd component=hello signature
params:
  value: string
```
