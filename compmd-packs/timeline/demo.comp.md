---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

Each example of `timeline` shows its source, then what it draws.

# Events along a line

Each `event` has a dot on the line, its date, a title, and what you write in it.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

::: timeline
::: event {date=1897 title="The harbour wall"}
Built in a single summer, of granite from the quarry above the town.
::: /event

::: event {date=1952 title="After the storm"}
The quay is rebuilt a metre higher, and the fish market moves to its end.
::: /event

::: event {date=2024 title="The lamps come back"}
All forty of them, restored, one post at a time from the chapel end.
::: /event
::: /timeline
````

# The current event

`tone=accent` fills its dot with the accent colour.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

::: timeline
::: event {date="March 2023" title="Taken down"}
The lamp posts go to the foundry for repair.
::: /event

::: event {date="October 2023" title="The first six"}
Back on the quay, by the lifeboat station.
::: /event

::: event {date="April 2024" title="The last of them" tone=accent}
Lit again on a spring evening, with a band.
::: /event
::: /timeline
````

# More to say, or only a date

An event holds paragraphs, links and lists; one with no title is its date and its text.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

::: timeline
::: event {date="Day 1" title="The harbour"}
The walk along [the old quay](#quay), then the fish market as the boats come in.

Stay for the auction if you can: it's over in twenty minutes.
::: /event

::: event {date="Day 2" title="The orchard"}
A long day. Bring:

- water, since there's no shop on the hill;
- a bag for the apples.
::: /event

::: event {date="Day 3"}
A day of rest, by the canal.
::: /event
::: /timeline
````

# From data

The events written from a data file, `milestones.yaml`, with `each`.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
  data:
    - from: ./milestones.yaml
      as: milestones
---

::: timeline
::: each {in=(md.data.milestones) as=m}
::: event {date=(m.date) title=(m.title)}
{{m.what}}
::: /event
::: /each
::: /timeline
````

# A quick timeline

A list, with no events: the list's markers on the line.

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
