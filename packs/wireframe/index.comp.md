---
md:
  import:
    - from: ./dist/wireframe.js
      names: [wireframe]
  export:
    pack: "@comp-markdown/wireframe"
    version: "0.1.0"
    names: [wireframe]
---

# @comp-markdown/wireframe

`wireframe` draws every ` ```wireframe ` block of a document that imports it, in the page's colour scheme; a reader without it sees the code. Written as a leaf, its `code` is a param, for source made from data: `:: wireframe[What it shows] {code=("…")}`.

```compmd component=wireframe signature
renders: wireframe
```
