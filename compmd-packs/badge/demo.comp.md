---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

Each example of `badge` shows its source, then what it draws.

# A badge in a sentence

Its text is the text in `[…]`, `subtle` by default.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

Search is faster from version :badge[2.1].
````

# Each tone

`subtle`, `strong` and `accent`.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The old search :badge[deprecated]{tone=subtle}, the map :badge[stable]{tone=strong}, the new search :badge[beta]{tone=accent}.
````

# In a heading

As tall as its text.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

## Saved searches :badge[new]{tone=accent}
````

# Its box changed by the use

As any box's: a border, square corners.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The walk is :badge[closed for repairs]{border=strong border-radius=none background=none} until May.
````

# Its tone from data

An expression.

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

# Your own badges, defined once

`labels.comp.md`, next to this demo, defines `bug`, `feature` and `chore`, each a `badge` with its colours, border and bold or italic text fixed. A document imports the file and uses them with nothing to configure.

````compmd {render=both}
---
md:
  import:
    - from: ./labels.comp.md
      names: [bug, feature, chore]
---

## Changes in 2.1

- :feature[new] Saved searches, from the search page.
- :feature[new] A map of the walks, each stop a pin.
- :bug[bug] The tide table showed yesterday's tides after midnight.
- :bug[bug] A walk's distance was in miles on a page set to kilometres.
- :chore[chore] The photos are smaller, so the pages load faster.
````
