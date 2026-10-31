# 5. Showing and repeating

Some parts of a page depend on what the frontmatter holds: a line that shows only while plots are free, a list with one item per job, a box per gardener. Two built-in containers do this, `if` and `each`. With them, the newsletter's lists come from its frontmatter, and next month's issue is mostly a change of values.

## The newsletter, from its lists

```md snippet=showing/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
jobs: [Turn the compost, Mend the gate by plot 4, Plant the garlic]
helpers: [Maya, Tomás, Jun]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
  - {number: 5, gardener: "", crop: ""}
news: []
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Work day

Saturday, November 7, from nine to noon. On the list:

::: each {in=(md.doc.jobs) as=job join=list}

1. {{job}}

::: /each

Thanks to last month's helpers:

::: each {in=(md.doc.helpers) as=helper join=inline sep=", " last=" and "}

**{{helper}}**

::: /each

## The plots

::: each {in=(md.doc.plots.filter(p, p.gardener != "")) as=p}

::: box {padding=s border=subtle border-radius=s}

**Plot {{p.number}}**: {{p.gardener}}, growing {{p.crop}}.

::: /box

::: /each

::: if {when=(md.doc.plots.exists(p, p.gardener == ""))}

:value[2 plots are free]{of=(string(size(md.doc.plots.filter(p, p.gardener == ""))) + " " + md.plural(size(md.doc.plots.filter(p, p.gardener == "")), "plot is", "plots are") + " free")} for next year. Ask Maya to take one.

:: #else

Every plot is taken for next year. Ask Maya to put you on the waiting list.

::: /if

::: each {in=(md.doc.news) as=item join=list}

- {{item}}

:: #before

## From the committee

:: #empty

No news from the committee this month.

::: /each
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/showing) in the playground.

The page shows the three jobs as one numbered list, a line thanking *Maya, Tomás and Jun*, a box for each of the three plots with a gardener, the line *2 plots are free for next year*, and *No news from the committee this month.*

## A list of records

The `plots` key is a list of records, one per plot, each with the same names: `number`, `gardener` and `crop`. In YAML, each item starts with `- `, and a record can be written in braces on one line. This is how a document holds a table's worth of values; Part II reads the same from a spreadsheet file.

## `if`

`if` shows its body when its `when` param is true:

```md
---
water_on: true
---

::: if {when=(md.doc.water_on)}

The taps work until November 15.

:: #else

The water is off for the winter.

::: /if
```

`when` is an expression that is true or false. What's after the `:: #else` line shows when it is false; leave that part out to show nothing. The `:: #else` line starts a *slot*, a named part of the container (chapter 3).

In the newsletter, `when` asks whether any plot has no gardener: `md.doc.plots.exists(p, p.gardener == "")`. `exists` goes through a list, calling each item `p`, and is true when the condition after the comma is true for at least one of them.

When only one value differs, a param computed with `? :` is shorter than two branches: `background=(md.doc.water_on ? "subtle" : "accent")`, as in chapter 4.

## `each`

`each` shows its body once for each item of a list. `in` is the list, and `as` gives each item a name to use in the body:

```md
---
jobs: [Turn the compost, Mend the gate, Plant the garlic]
---

::: each {in=(md.doc.jobs) as=job}

**Job:** {{job}}

::: /each
```

This shows three paragraphs, each starting **Job:**. With a list of records, a dot reads a value of the record: `{{p.gardener}}`.

The `as` name is yours to choose. Make it one word, or words joined by `_`, such as `job` or `plot_number`.

### One list, or one paragraph

Each time round, the body makes its own blocks. To make them one list instead of three lists of one item, write the body as one list item and add `join=list`. A numbered list then counts on, 1, 2, 3, from one item to the next.

To make them one paragraph, write the body as a paragraph and add `join=inline`. `sep` goes between the items, a space unless you say otherwise, and `last` between the last two, so `sep=", " last=" and "` gives *Maya, Tomás and Jun*.

Without `join`, every time round adds its blocks to the page, one after another, as the boxes of the plots do. An `each` around a box gives a box per item; Part III puts such boxes side by side, as cards.

### Before and empty

Two slots handle the edges. What's under `:: #before` shows once, above the items, but only when there is at least one; it's the place for a list's heading. What's under `:: #empty` shows when there are none. In the newsletter, `news` is an empty list, `[]`, so the heading *From the committee* doesn't show, and the empty message does.

Slot lines start at the container's own level, not indented under the list item, which would put them inside it.

### Which time round

Inside the body, `md.loop` says where the list is:

```md
---
jobs: [Turn the compost, Mend the gate, Plant the garlic]
---

::: each {in=(md.doc.jobs) as=job}

**Job {{md.loop.number}} of {{md.loop.count}}:** {{job}}

::: /each
```

| Path | Is |
| --- | --- |
| `md.loop.number` | 1 for the first item, 2 for the next, and so on |
| `md.loop.count` | how many items there are |
| `md.loop.first`, `md.loop.last` | true for the first item, or for the last |
| `md.loop.index` | 0 for the first item, as list positions in expressions count |

## Choosing items

`in` is an expression, so it can choose and order items before `each` sees them. The newsletter shows only plots with a gardener: `md.doc.plots.filter(p, p.gardener != "")`. `filter` keeps the items for which the condition is true.

A few more, all written after a list:

| Written | Gives |
| --- | --- |
| `md.doc.plots.filter(p, p.crop == "garlic")` | the items for which the condition is true |
| `md.doc.plots.exists(p, p.gardener == "")` | true when the condition is true for at least one item |
| `md.doc.plots.all(p, p.gardener != "")` | true when it is true for every item |
| `md.doc.plots.map(p, p.gardener)` | a list of one value from each item: the gardeners |

And a few functions that take a list, for sorting and counting: `md.sortBy(md.doc.plots, "crop")` sorts by a name of the records, `md.reverse(…)` turns a list around, `md.slice(list, 0, 3)` takes the first three, and `size(…)` counts. Chapter 8 lists them all.

## What `each` can't do

`each` makes whole blocks, so it can't make the rows of a table: a Markdown table is one block. A table from a list comes from a component that makes tables, such as one from a pack (chapter 16).

Next: Part II, [data from files](README.md#contents), where the lists come from spreadsheets and files instead of the frontmatter.
