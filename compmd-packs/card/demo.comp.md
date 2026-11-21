---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

# card

Each example shows its source, then what it draws.

**A card with an image, its text and a footer:**

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: card
### The harbour at dusk

A walk along the old quay, an hour before the boats come in.

:: #image
![Boats at the quay, the lamps just lit](harbour.svg)

:: #footer
[Read the walk](#harbour)
::: /card
````

**No image and no footer:** the card leaves them out.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: card
### Along the canal

Flat, quiet, and twelve bridges. No picture yet.
::: /card
````

**Its box changed by the use,** as any box's: a tinted background, no border, more padding, a width:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: card {background=subtle border=none padding=l width=20rem}
### The old orchard

Apples in October, and a bench at the top of the hill.

:: #image
![Rows of apple trees on a slope](orchard.svg)
::: /card
````

**Cards in a grid,** as many side by side as fit at 14rem, each as tall as the tallest in its row:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: grid {min=14rem gap=m}
::: card
### The harbour at dusk

An hour before the boats come in.

:: #image
![Boats at the quay, the lamps just lit](harbour.svg)
::: /card

::: card
### The old orchard

Apples in October, and a bench at the top of the hill, where the town shows its roofs.

:: #image
![Rows of apple trees on a slope](orchard.svg)
::: /card

::: card
### Along the canal

Twelve bridges.
::: /card
::: /grid
````
