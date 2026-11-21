---
md:
  export:
    pack: "@comp-markdown/columns"
    version: "0.1.0"
    names: [columns]
---

# @comp-markdown/columns

`columns` lays out two or three columns side by side: a recipe's ingredients beside its method, a newsletter's two stories, a page's text with a note at its end.

```md
::: columns {split=wide-end}
**Ingredients**

- 4 apples
- 100 g butter

:: #end
**Method**

Peel and slice the apples, then...
::: /columns
```

What you write first is the first column; `:: #middle` starts a middle one and `:: #end` the last, so a column holds any blocks with no `box` around them. `split` sets the widths: `even` (the default), `wide-start`, `wide-end`, or `wide-middle` with three. The columns are side by side from the `sm` breakpoint up (480 px of their box) and stack in order below it, as a plain Markdown reader shows them. `gap` is the space between them (`m`). Start and end follow the text's direction, so in right-to-left text the first column is on the right. A use changes the box as any box's, `{padding=m border=subtle}`.

```compmd component=columns {display=grid gap=(gap) sm:grid-template-columns=(split == "wide-start" ? (md.slots.middle ? "2fr 1fr 1fr" : "2fr 1fr") : split == "wide-end" ? (md.slots.middle ? "1fr 1fr 2fr" : "1fr 2fr") : split == "wide-middle" ? "1fr 2fr 1fr" : (md.slots.middle ? "1fr 1fr 1fr" : "1fr 1fr"))}
params:
  split:
    type: even | wide-start | wide-middle | wide-end
    default: even
    doc: Which column is wider, twice the others; `wide-middle` with three
  gap:
    type: space
    default: m
    doc: The space between the columns, and between them stacked
slots: [default, middle, end]
---
::: box
{{md.slot}}
::: /box

::: if {when=(md.slots.middle)}
::: box
{{md.slot.middle}}
::: /box
::: /if

::: box
{{md.slot.end}}
::: /box
```
