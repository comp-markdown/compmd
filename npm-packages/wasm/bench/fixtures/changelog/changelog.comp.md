---
repo: acme/rayplay
md:
  data:
    - from: ./releases.csv
      as: releases
      schema: {version: string, date: date}
    - from: ./changes.csv
      as: changes
      schema: {release: string, text: string, pr: number, type: breaking | feature | fix}
---

```compmd component=changes {display=contents}
params:
  release: string
  kind: breaking | feature | fix
  title: string
---
::: if {when=(md.data.changes.exists(c, c.release == release && c.type == kind))}
# {{title}}

::: each {in=(md.data.changes.filter(c, c.release == release && c.type == kind)) as=c join=list}
- {{c.text}} ([#{{c.pr}}](https://github.com/{{md.doc.repo}}/pull/{{c.pr}}))
::: /each
::: /if
```

# RayPlay changelog

::: each {in=(md.data.releases) as=r}
## {{r.version}}, :value[October 21, 2026]{of=(md.date(r.date, "long"))}

:: changes[Breaking changes] {release=(r.version) kind=breaking title="Breaking changes"}

:: changes[Features] {release=(r.version) kind=feature title="Features"}

:: changes[Fixes] {release=(r.version) kind=fix title="Fixes"}
::: /each
