---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
---

Each example of `timeline` shows its source, then what it draws.

# A timeline of years

A list, each item a date and what happened.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
---

::: timeline
- **1897**: the harbour wall is built.
- **1952**: the quay is rebuilt after the storm.
- **2024**: the lamps along the quay come back.
::: /timeline
````

# Days of a trip, with more to say

An item holds what a list item holds: paragraphs, a link, a nested list.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
---

::: timeline
- **Day 1**: the harbour, then [the walk along the quay](#quay).
- **Day 2**: the orchard, a long day.

  Bring water: there's no shop on the hill.
- **Day 3**: the canal, and its twelve bridges:
  - the iron bridge;
  - the lock keeper's bridge.
::: /timeline
````

# Its box changed by the use

A fainter line, more room beside it.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
---

::: timeline {border-inline-start=subtle padding-inline-start=l}
- **May 3**: the first boats are back.
- **June 21**: the longest evening on the quay.
::: /timeline
````

# From data

The list written from a data file, `milestones.yaml`, with `each`; `join=list` makes its items one list.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
  data:
    - from: ./milestones.yaml
      as: milestones
---

::: timeline
::: each {in=(md.data.milestones) as=m join=list}
- **{{m.date}}**: {{m.what}}
::: /each
::: /timeline
````
