---
md:
  version: 1
  import:
    - from: "@myorg/ui@1"
      names: [card, badge]
    - from: ./chart.jsx
      names: [chart]
title: Release notes
version: "1.4"
changes:
  - {text: "New wgpu renderer", pr: 412, breaking: false}
  - {text: "Config format 2", pr: 398, breaking: true}
---

# {{md.doc.title}} :badge[beta]{tone=info}

RayPlay {{md.doc.version}} ships a new renderer.

```compmd component=callout {padding=m border-radius=m border=subtle background=subtle class=("callout-" + kind)}
params:
  kind: tip | warning = tip
slots: [default]
---
{{md.slot}}
```

::: grid {gap=l}
::: + md:grid-template-columns="2fr 1fr"

::: card {title="Setup" icon=rocket}
Install with `cargo install rayplay`, then run `rayplay init`.

:: #footer
Takes about 2 minutes.
::: /card

:: chart[Weekly installs: up 18% since 1.3] {src=installs.csv type=bar}

::: /grid

## What changed

::: each {in=(md.doc.changes) as=c join=list}
- {{c.text}} ([#{{c.pr}}](https://github.com/acme/rayplay/pull/{{c.pr}}))
::: /each

::: if {when=(md.doc.changes.exists(c, c.breaking))}
::: callout {kind=warning}
The 1.3 config format is no longer read. Run `rayplay migrate` first.
::: /callout
::: /if
