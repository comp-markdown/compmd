---
title: Definitions and slots
tasks:
  - {title: Draft the spec, status: done, owner: Ana}
  - {title: Write the parser, status: doing, owner: Kenji}
  - {title: Build the editor, status: doing, owner: Priya}
  - {title: Publish the pack, status: todo, owner: Ana}
lanes: [todo, doing, done]
md:
  version: 1
---

# {{md.doc.title}}

A component written in CompMD is a fence with a signature, `---`, and a body. Its params are typed, its slots are named, and the fence's own params style its box.

```compmd component=card {padding=m border=subtle border-radius=m}
params:
  title: string
  tone: plain | warning = plain
slots: [default, footer]
---
### {{title}}

{{md.slot}}

::: if {when=(tone == "warning")}
**Read this first.**
::: /if

{{md.slot.footer}}
```

## Using it

A container fills the default slot with its body, and a named slot from its `:: #name` line on:

::: card {title="Plain"}
The body is the default slot.

:: #footer
*The footer slot.*
::: /card

::: card {title="Careful" tone=warning}
A param picks what the body shows.
::: /card

As a leaf, a card has no body; the brackets hold its fallback:

:: card[A card without a body] {title="Leaf"}

## Composing

Definitions use each other, and pass their params on:

```compmd component=person {display=contents}
params:
  name: string
  role: string
---
::: card {title="{{name}}"}
{{role}}
::: /card
```

::: grid {min=12rem gap=m}
:: person[Ana Lima] {name="Ana Lima" role="Editor"}
:: person[Kenji Sato] {name="Kenji Sato" role="Parser"}
::: /grid

`person` has `display=contents`, so it adds no box of its own: the grid places each card.

## Template slots

A template slot renders once per element of an array param, with the element under the name the caller picks. Here `lane` takes the tasks of one status, and the caller writes each task's template:

```compmd component=lane {display=flex flex-direction=column gap=s padding=m border=subtle border-radius=m}
params:
  status: string
  rows: array
  as: name
slots:
  default: {each: rows, as: as}
---
#### {{status}}

{{md.slot}}
```

::: grid {md:grid-template-columns="1fr 1fr 1fr" gap=m}
::: each {in=(md.doc.lanes) as=s}
::: lane {status="{{s}}" rows=(md.doc.tasks.filter(t, t.status == s)) as=t}
**{{t.title}}**, {{t.owner}}
::: /lane
::: /each
::: /grid

`each` does the same for any content, and its own `empty` slot shows when there is nothing:

::: each {in=(md.doc.tasks.filter(t, t.owner == "Nobody")) as=t join=list}
- {{t.title}}

:: #empty
No task belongs to Nobody.
::: /each
