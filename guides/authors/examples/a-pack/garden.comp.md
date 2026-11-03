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

## Next year's beds

Each bed moves one step along the rotation every spring, so nothing grows where it grew last year, and the soil gets a rest from each crop.

:: mermaid[Beans, then cabbages, then roots, then potatoes, then beans again] {code=("graph LR; " + md.doc.rotation.join(" --> ") + " --> " + md.doc.rotation[0])}

Your bed's crop next year is the one after this year's.
