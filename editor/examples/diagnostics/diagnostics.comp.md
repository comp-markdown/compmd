---
title: Diagnostics and their fixes
md:
  version: 1
---

# {{md.doc.title}}

This document has five mistakes. Each one is a diagnostic under the text: its code, its place, which selects it, and what's wrong. A diagnostic that suggests a name or a value has a button that makes the change. Fix them all, and the list is empty.

A mistake never stops the page: the part it's in renders as well as it can, and the rest of the page as usual.

## A component that doesn't exist (F002)

`valeu` is two letters swapped in `value`, so this line shows the fallback in brackets, not the date the value formats: today is :valeu[a day in 2026]{of=(md.date(md.today(), "long"))}.

## A param that doesn't exist (F008)

`borde` is one letter off `border`, so the box has no border:

::: box {padding=m borde=subtle}
A box with a border, once the param is fixed.
::: /box

## A value a param doesn't take (F009)

`border-radius` takes `none`, `s`, `m`, `l`, `full` or a length. `ful` is close to `full`:

::: box {padding=m border=subtle border-radius=ful}
A pill-shaped box, once the value is fixed.
::: /box

## A value a document's own component doesn't take (F009)

```compmd component=badge {padding="2px 8px" border-radius=full background=subtle}
params:
  tone: info | warning | danger = info
---
**{{tone}}**
```

`badge` takes `info`, `warning` or `danger` for `tone`:

:: badge[warning] {tone=warnign}

## A slot that doesn't exist (F006)

```compmd component=card {padding=m border=subtle border-radius=m}
slots: [default, footer]
---
{{md.slot}}

{{md.slot.footer}}
```

`card` has a `footer` slot, not a `fotter` one:

::: card
The body.

:: #fotter
*The footer, once its name is fixed.*
::: /card
