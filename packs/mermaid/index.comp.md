---
md:
  import:
    - from: ./dist/mermaid.js
      names: [mermaid]
  export:
    pack: "@comp-markdown/mermaid"
    version: "0.1.1"
    names: [mermaid]
---

# @comp-markdown/mermaid

`mermaid` draws every ` ```mermaid ` (or ` ```mmd `) block of a document that imports it, with Mermaid 12.2.0; a reader without it sees the code. Written as a leaf, its `code` is a param, for a diagram made from data: `:: mermaid[Our process] {code=("graph LR; " + md.doc.steps.join(" --> "))}`.

`theme` is `auto` by default: Mermaid's dark theme when the page's colour scheme is dark, its default theme otherwise. A block Mermaid can't read shows its code, and the host reports what Mermaid found.

```compmd component=mermaid signature
renders: [mermaid, mmd]
params:
  theme: auto | default | dark | neutral | forest = auto
```
