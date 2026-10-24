---
title: RayPlay 1.4
version: "1.4"
launch: "2026-11-02"
changes:
  - {text: "New wgpu renderer", pr: 412, type: feature}
  - {text: "Config format 2", pr: 398, type: breaking}
stack:
  - {name: Rust, url: "https://www.rust-lang.org"}
  - {name: wgpu, url: "https://wgpu.rs"}
  - {name: WebAssembly, url: "https://webassembly.org"}
md:
  version: 1
  lang: en
  tz: Europe/Berlin
  syntax: [wiki-links]
  import:
    - from: "@acme/charts@1.4.2"
      integrity: sha512-3q9wHVm8bVY3oEhTyWJOR4kUqMVfF0Pn3eW0v1jYQ2Q1…
      names: [chart]
    - from: ./board.jsx
      names: [board]
  export:
    names: [card]
  data:
    - from: ./installs.csv
      as: installs
      schema: {date: date, region: string, count: number}
    - from: ./team.yaml
      as: team
    - from: https://status.rayplay.dev/today.json?region={{md.input.region}}
      as: status
      cache: 10m
      fallback: ./status.json
    - from: host:tasks?project=rayplay
      as: tasks
      schema: {title: string, status: todo | doing | done, due: date}
  inputs:
    region: all | eu | us = all
    since: date = (md.addDays(md.today(), -30))
    highlight: string = ""
---

# {{md.doc.title}}

RayPlay {{md.doc.version}} ships on :value[November 2, 2026]{of=(md.date(md.doc.launch, "long"))}. Read the [[Upgrade guide|upgrade guide]] first.[^migrate]

[^migrate]: Config files from 1.3 need `rayplay migrate`.

> [!NOTE]
> This page renders live data. Write `\{{name}}` to show braces in prose.

```compmd component=card {padding=m border="1px solid" border-radius=8px class=("card-" + kind)}
params:
  kind: note | warning = note
  title: string
slots: [default, footer]
---
# {{title}}

{{md.slot}}

{{md.slot.footer}}
```

```compmd component=board signature
params:
  rows: array
  by: string
  as: name
slots:
  default: {each: rows, as: as}
```

```jsx component=sparkline runtime=react@19
export default function Sparkline({ values, compmd }) {
  const max = Math.max(...values);
  const points = values.map((v, i) => `${i * 10},${20 - (v / max) * 20}`).join(" ");
  return (
    <svg role="img" aria-label={compmd.fallback.text} viewBox={`0 0 ${(values.length - 1) * 10} 20`}>
      <polyline points={points} fill="none" stroke="currentColor" />
    </svg>
  );
}
```

```compmd component=sparkline signature
params:
  values: array
```

## What changed

::: each {in=(md.doc.changes) as=c join=list}
- {{c.text}} ([#{{c.pr}}](https://github.com/acme/rayplay/pull/{{c.pr}}))

:: #empty
Nothing changed in this release.
::: /each

::: if {when=(md.doc.changes.exists(c, c.type == "breaking"))}
::: card {kind=warning title="Breaking changes"}
Run `rayplay migrate` before starting 1.4.

:: #footer
It takes about two minutes.
::: /card

:: #else
This release is backward compatible.
::: /if

## Installs

:: select-input[Region: all] {bind=region}
:: date-input[Since: 30 days ago] {bind=since}

::: grid {#installs gap=m}
::: + md:grid-template-columns="2fr 1fr" lg:gap=l

:: chart[Daily installs since {{md.input.since}}] {rows=(md.data.installs.filter(i, i.date >= md.input.since && (md.input.region == "all" || i.region == md.input.region))) type=bar selected=highlight}

::: stack {gap=s sticky=top show=md}
**Total:** :value[12,480]{of=(md.number(md.sum(md.data.installs.map(i, i.count)), 0))} :sparkline[rising]{values=[3,5,8,13]}

**Status:** {{md.data.status.summary}}
::: /stack

::: /grid

| Region | Since |
| ------ | ----- |
| {{md.input.region}} | {{md.input.since}} |

## Team

::: grid {min=14rem gap=m}
::: each {in=(md.data.team) as=p}
::: card {title="{{p.name}}" .person}
{{p.role}}, since {{p.since}}
::: /card
::: /each
::: /grid

## Tasks

::: board {rows=(md.data.tasks) by=status as=t}
**{{t.title}}**, due :value[Nov 2, 2026]{of=(md.date(t.due, "medium"))}
::: /board

## Built with

::: row {gap=s justify-content=space-between align-items=center}
::: box
::: each {in=(md.doc.stack) as=s join=inline sep=", " last=" and "}
[{{s.name}}]({{s.url}})
::: /each
::: /box

::: box {lang=de justify-self=end}
„Endlich flüssig.“ Frame time $p_{95} < 16\,\text{ms}$.
::: /box
::: /row
