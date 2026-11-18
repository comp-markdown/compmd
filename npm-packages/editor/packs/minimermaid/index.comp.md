---
md:
  import:
    - from: ./dist/minimermaid.js
      names: [minimermaid]
  export:
    pack: "@preview/minimermaid"
    version: "0.1.0"
    names: [minimermaid]
---

# @preview/minimermaid

`minimermaid` draws every ` ```mermaid ` block of a document that imports it, by hand: Mermaid's simplest flowcharts, `graph LR` or `graph TD`, then chains of steps joined by `-->`, each `id` or `id[label]`, and nothing else of Mermaid. A reader without it sees the code. Written as a leaf, its `code` is a param, for source made from data: `:: minimermaid[What it shows] {code=("graph LR; " + md.doc.steps.join(" --> "))}`.

```compmd component=minimermaid signature
renders: mermaid
params:
  theme: default | dark | neutral = default
```
