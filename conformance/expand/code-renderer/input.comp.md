---
md:
  import:
    - from: "@acme/diagrams"
      names: [mermaid]
steps: [Draft, Review]
---

```compmd component=shout
renders: shout
---
**{{code}}**
```

A block rendered by the host's renderer, one in another name of its language, the same component as a leaf, a block a
`compmd` definition renders, and a block no renderer renders.

```mermaid {theme=dark alt="Draft, then review"}
graph LR; Draft --> Review
```

```mmd {alt="A, then B"}
graph LR; A --> B
```

:: mermaid[Draft, then review] {code=("graph LR; " + md.doc.steps.join(" --> "))}

```shout {alt="Hello"}
hello
```

```sh
echo plain
```
