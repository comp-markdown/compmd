# 14. Template slots

Some components show a list: a bed with its plots, a calendar with its events. A *template slot* lets such a component take the list as a param and the way to show each item as a slot, written by you. This chapter gives the newsletter a `bed` component that shows each bed's plots, with a count and a message for a bed that has none.

## The newsletter, by bed

````md snippet=template-slots/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
beds: [North, South, East]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic, bed: North}
  - {number: 2, gardener: Tomás Reyes, crop: kale, bed: North}
  - {number: 3, gardener: Jun Park, crop: beans, bed: South}
  - {number: 4, gardener: Priya Nair, crop: leeks, bed: South}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=bed {display=flex flex-direction=column gap=s padding=m border=subtle border-radius=m}
params:
  title: string
  plots: array
  as: name
slots:
  default: {each: plots, as: as}
  empty: {}
---
# The {{title}} bed

:value[2 plots]{of=(string(size(plots)) + " " + md.plural(size(plots), "plot", "plots"))}

{{md.slot}}

::: if {when=(size(plots) == 0)}

{{md.slot.empty}}

::: /if
```

## The beds

::: grid {min=14rem gap=m}

::: each {in=(md.doc.beds) as=b}

::: bed {title="{{b}}" plots=(md.doc.plots.filter(p, p.bed == b)) as=plot}

**Plot {{plot.number}}**: {{plot.gardener}}, growing {{plot.crop}}.

:: #empty

No plots here yet. Ask Maya for one.

::: /bed

::: /each

::: /grid
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/template-slots) in the playground.

The page shows three boxes side by side. *The North bed* says *2 plots* and has a line for each of Maya's and Tomás's plots; *The South bed* has Jun's and Priya's; *The East bed* says *0 plots* and *No plots here yet.* Add a plot with `bed: East`, and it moves into the third box.

## A slot for each item

The signature says which slot is a template, and for which list:

```text
params:
  title: string
  plots: array
  as: name
slots:
  default: {each: plots, as: as}
  empty: {}
```

- `plots` is an `array` param: a list, here of records.
- `as` is a `name` param: the name the use gives each item. Its value is a name, not text or an expression.
- Under `slots`, each slot now has its options, in braces. `{each: plots, as: as}` makes the default slot a template: it shows once for each item of `plots`, with the item under the name `as` holds. `empty: {}` is an ordinary slot, with no options.

The params can have other names: `{each: rows, as: call}` works with `rows: array` and `call: name`. Calling them `rows` and `as`, or after what they hold, keeps uses easy to read.

## Writing the template

A use passes the list, and picks the name:

```text
::: bed {title="North" plots=(md.doc.plots.filter(p, p.bed == "North")) as=plot}

**Plot {{plot.number}}**: {{plot.gardener}}, growing {{plot.crop}}.

::: /bed
```

The body you write is the template, written once and shown once per plot, with `{{plot.number}}` and `{{plot.gardener}}` that plot's. Inside it, `md.loop` says which time round it is, as in `each` (chapter 5): `{{md.loop.number}} of {{md.loop.count}}`.

The name is yours: `as=plot` here, `as=p` or `as=entry` as well. It is one word, or words joined by `_`, as `each`'s `as` is. Some words are taken by expressions, such as `in` or `true`, and can't be used (`F013`). An expression, `as=(plot)`, is `F009`: the name is written as it is.

The name is known in the template only. In another slot, such as `empty`, `{{plot.number}}` is a diagnostic (`F014`), since there is no plot there.

A template slot that isn't the default one is filled from its slot line, as any named slot is: a slot declared `item: {each: rows, as: as}` takes its template after `:: #item`.

## Where the items go

In the body, `{{md.slot}}` places every item's template, one after another, in the list's order. The body puts what goes once around them: the title above, and the message for an empty bed below. It can also use the list itself in expressions, as `size(plots)` counts it, both for the count and to show `empty` only when there are no plots.

The fence styles the bed's box as a column (`display=flex flex-direction=column`), with a gap between its blocks. So the heading, the count and each plot's line are spaced evenly.

A component written in CompMD places the items together, where `{{md.slot}}` is. A component written in code, such as a calendar or a board from a pack, can place each item where it belongs, in its day or its column. You write the template the same way for both.

## Choosing the items

Which items a component shows is the use's to say, with an expression: `plots=(md.doc.plots.filter(p, p.bed == b))` gives the bed only its own plots. Sorting goes the same way, with `md.sortBy` (chapter 5).

A value worked out for each item goes in the list first, with `map`, which makes a new record per item:

```text
plots=(md.doc.plots.map(p, {"number": p.number, "gardener": p.gardener == "" ? "nobody yet" : p.gardener}))
```

## Template slot or `each`

`each` repeats any content once per item, and a component can go inside it: an `each` around a `plot-card` gives a card per plot, as in chapter 12. A template slot is for a component that is itself about the list:

| Use | When |
| --- | --- |
| `each` around a component | each item is a component of its own, and what goes around the list is yours to write each time |
| a template slot | the component frames the list, with its heading, its count, its layout, or its message when the list is empty |

The newsletter uses both: `each` makes a bed per name in `beds`, and each bed's template slot shows its plots. In fact `each` is itself a component with a template slot, `default: {each: in, as: as}`, and the slots `before` and `empty`.

A plain Markdown viewer shows a template slot's template once, as written, as it shows an `each`'s.

Next: [signatures and fallbacks](15-signatures-and-fallbacks.md), for every type a param can have, components kept in a file of their own, and what shows when a component can't be drawn.
