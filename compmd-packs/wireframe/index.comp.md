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

`wireframe` draws every ` ```wireframe ` block of a document that imports it: a screen written as an indented outline, one part per line ([the language](./LANGUAGE.md)), drawn as HTML and CSS in the page's colour scheme. A reader without it sees the code. Written as a leaf, its `code` is a param, for a screen made from data: `:: wireframe[The steps as tabs] {code=("window:\n  tabs:\n" + …)}`.

A source the language can't read shows its code, and the host reports its first mistake, with its line and column in the block.

```compmd component=wireframe signature
renders: wireframe
```
