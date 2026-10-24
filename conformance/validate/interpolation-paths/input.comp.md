---
title: Release
team: []
md:
  data:
    - from: ./team.yaml
      as: people
  inputs:
    region: all | eu = all
---

# {{md.doc.title}} {{md.version}} {{md.lang}} {{md.data.people.name}} {{md.input.region}}

::: each {in=(md.doc.team) as=p}
{{p.name}} is number {{md.loop.number}}

:: #empty
Nobody, so no {{p.name}} here.
::: /each

[Profile](https://example.com/{{md.doc.title}}) {{md.doc.missing}} {{md.loop.index}} {{md.data.nothing}} {{md.input.city}} {{md.slot}}
