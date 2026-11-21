---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

# sticky-note

Each example shows its source, then what it draws.

**A note in the flow,** where it is written:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: sticky-note
Bring the tide table: the quay floods at high water.
::: /sticky-note
````

**Pinned in its box from `sm` up,** over the box's end, and in the flow on a phone. The box is as tall as its text; the text leaves room for the note:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: box {border=subtle border-radius=m padding=m}
## The harbour walk

::: box {sm:max-width=55%}
Start at the lifeboat station and follow the quay to the lighthouse. The lamps come on an hour before the boats come in, and the fish market opens when they do.

Back along the old wall, past the chapel, to the steps up to the town.
::: /box

::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=3rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
::: /box
````

**Its box changed by the use,** as any box's: tilted the other way, a quieter tint, narrower:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: sticky-note {rotate=3deg background=subtle width=10rem}
Closed on Mondays.
::: /sticky-note
````
