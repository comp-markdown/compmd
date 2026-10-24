```compmd component=card {.card padding=s border=subtle md:padding=m}
params:
  title: string
slots: [default]
---
**{{title}}**

{{md.slot}}
```

```compmd component=inset-card {padding-inline=l}
slots: [default]
---
{{md.slot}}
```

```compmd component=section
params:
  title: string
slots: [default]
---
**{{title}}**

{{md.slot}}
```

```compmd component=cards
params:
  rows: array
---
::: each {in=(rows) as=r}
::: card {title=(r)}
::: /card
::: /each
```

```compmd component=tiles {display=contents}
params:
  rows: array
---
::: each {in=(rows) as=r}
::: card {title=(r)}
::: /card
::: /each
```

::: card {title="Wide" .wide padding=l background=subtle}
The caller's padding wins at every width, the classes add up.
::: /card

::: card {title="From md" md:padding=l}
The definition's padding below md, the caller's from md up.
::: /card

::: inset-card {padding=s}
The caller's padding covers the definition's padding-inline.
::: /inset-card

::: inset-card {padding-inline-start=m}
The definition's padding-inline is split; its start side is the caller's.
::: /inset-card

::: row {height=8rem}
::: box {inset-inline-start=16px}
Placed in the row's own box, which has the height.
::: /box
::: /row

::: section {title="Changes"}
A box, with or without params.
::: /section

:: cards[One card] {rows=(["One"]) .list}

:: cards[Two cards] {rows=(["One", "Two"]) .list}

::: grid {min=10rem}
:: tiles[Two tiles] {rows=(["One", "Two"])}
::: /grid
