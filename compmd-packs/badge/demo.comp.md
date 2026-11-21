---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

# badge

Each example shows its source, then what it draws.

**A badge in a sentence,** its text the text in `[…]`, `subtle` by default:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

Search is faster from version :badge[2.1].
````

**Each `tone`:** `subtle`, `strong` and `accent`:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The old search :badge[deprecated]{tone=subtle}, the map :badge[stable]{tone=strong}, the new search :badge[beta]{tone=accent}.
````

**In a heading,** as tall as its text:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

## Saved searches :badge[new]{tone=accent}
````

**Its box changed by the use,** as any box's: a border, square corners:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The walk is :badge[closed for repairs]{border=strong border-radius=none background=none} until May.
````

**Its tone from data,** an expression:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
  data:
    - from: ./stops.yaml
      as: stops
---

::: each {in=(md.data.stops) as=stop}
{{stop.name}} :badge[{{stop.status}}]{tone=(stop.status == "open" ? "accent" : "subtle")}
::: /each
````
