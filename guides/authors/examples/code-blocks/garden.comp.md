---
md:
  version: 1
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
title: Elm Street Garden
issue: November 2026
rotation: [Beans, Cabbages, Roots, Potatoes]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Where your scraps go

The green bin by the gate takes your kitchen scraps all winter. This is their way back to the beds:

```mermaid {theme=forest alt="Kitchen scraps go in the green bin, then on the compost heap; turned on work days, the compost goes on the beds in spring"}
graph LR
  Scraps[Kitchen scraps] --> Bin[Green bin by the gate]
  Bin --> Heap[Compost heap]
  Heap -->|turned on work days| Compost
  Compost -->|in spring| Beds[The beds]
```

No meat, no bread, nothing cooked: they bring rats.

## Next year's beds

:: mermaid[Beans, then cabbages, then roots, then potatoes, then beans again] {code=("graph LR; " + md.doc.rotation.join(" --> ") + " --> " + md.doc.rotation[0])}
