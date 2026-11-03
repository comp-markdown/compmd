# 9. Boxes, rows, stacks and grids

So far the newsletter's blocks come one under another, as in any Markdown page. This chapter puts them side by side: the notices in a stack, the work day's jobs in a row, and the plots as cards in a grid, as many across as the page has room for. Four built-in containers do it: `box`, `stack`, `row` and `grid`.

## The newsletter, laid out

```md snippet=layout/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
jobs: [Turn the compost, Mend the gate by plot 4, Plant the garlic]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
  - {number: 5, gardener: "", crop: ""}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: stack {gap=s}

::: box {padding=m border-radius=m background=accent}

**The water goes off on November 15.** Empty your cans and hoses before then.

::: /box

::: box {padding=m border-radius=m background=subtle}

**The tools go back in the shed** after the work day. Maya has the key.

::: /box

::: /stack

## Work day

Saturday, November 7, from nine to noon. On the list:

::: row {gap=s}

::: each {in=(md.doc.jobs) as=job}

::: box {padding="s m" border=subtle border-radius=full}

{{job}}

::: /box

::: /each

::: /row

## The plots

::: grid {min=12rem gap=m}

::: each {in=(md.doc.plots) as=p}

::: box {padding=m border=subtle border-radius=m background=(p.gardener == "" ? "accent" : "none")}

### Plot {{p.number}}

::: if {when=(p.gardener != "")}

{{p.gardener}}, growing {{p.crop}}.

:: #else

Free for next year. Ask Maya.

::: /if

::: /box

::: /each

::: /grid
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/layout) in the playground. The two notices sit one under the other with a small gap, and the three jobs are pills side by side. Drag the bar between the text and the page: with the page narrow, the pills wrap onto a second line and the plots are one card per row; as it widens, two, three, then all five cards fit across, and the free plots stand out in the accent colour.

## The four layouts

| Container | Lays out its blocks | Its own params |
| --- | --- | --- |
| `box` | one under another, as the page does | `display` and the others below, which the three after it set for you |
| `stack` | one under another, with a gap between them | `gap`, `align-items` |
| `row` | side by side, wrapping onto the next line when there isn't room | `gap`, `align-items`, `justify-content`, `flex-wrap` |
| `grid` | in columns: as many as fit (`min`), or the columns you give | `min`, `gap`, `align-items`, `grid-template-columns`, `grid-template-rows`, `grid-template-areas` |

Each of them is also a box, with the common params of chapter 3: `padding`, `border`, `background` and the others work on a row or a grid as on a box.

## Items

A layout lays out its *items*: each block directly in it is one. A paragraph is an item, and so is a heading, a list, a table, or a container in it. In the newsletter, each notice is a box, so the stack has two items; the `each` in the row makes three boxes, so the row has three. `each` and `if` add their blocks to the layout around them, as if you had written them there.

To keep several blocks together as one item, put them in a box. Without the box, a card's heading and its line would be two items, side by side in a row or in cells of their own in a grid:

```md
::: row {gap=m}

::: box {padding=m border=subtle}

### Plot 1

Maya Okafor, growing garlic.

::: /box

::: box {padding=m border=subtle}

### Plot 2

Tomás Reyes, growing kale.

::: /box

::: /row
```

## `stack`

A stack lays its items one under another, as the page does, with `gap` between them. Its use is the gap: a page puts its own space around paragraphs and headings, but a box has none, so boxes one after another touch. In a stack they don't.

`align-items` says how wide its items are: `stretch`, the default, makes each as wide as the stack; `start`, `center` and `end` make each as wide as its content, at the start, the middle or the end.

## `row`

A row lays its items side by side, each as wide as its content. When they don't fit, the row wraps onto the next line, as words do; with `flex-wrap=nowrap` it stays on one line, and its items shrink to fit.

- `align-items` lines items of different heights up: `stretch`, the default, makes them all as tall as the tallest; `start`, `center` and `end` line them up at the top, the middle or the bottom.
- `justify-content` says where the items go when there is room left on the line: `start` (the default), `center`, `end`, `between` for the space between them, and `around` for space around each of them. `space-between` and `space-around`, CSS's names for the last two, work too.

A row of two items with `justify-content=between` puts one at each end, as a title and a date in a heading:

```md
::: row {justify-content=between align-items=center}

**Elm Street Garden**

November 2026

::: /row
```

## `grid`

A grid lays its items in columns. There are two ways to say which:

- **As many as fit.** `min=12rem` makes columns at least 12rem wide, as many as the grid has room for, and shares what is left between them. This is the grid for cards: a wide page has five across, a phone one. It is the grid in the newsletter.
- **The columns you give.** `grid-template-columns="2fr 1fr"` makes two columns, the first twice as wide as the second. `fr` is a share of the room left; a column can also be a length, `12rem`, or `auto`, as wide as its content. Each item takes the next cell, row by row.

A grid with neither has one column, so its items come one under another; chapter 10 gives it columns from a width up. `gap` is the space between the cells, and `align-items` lines up items in a row of cells as in a `row`. `grid-template-rows` and `grid-template-areas`, which place items in cells of their own, are in chapter 11.

## Gaps and sizes

`gap` takes the same sizes as `padding`: `none`, `s`, `m`, `l`, or a length such as `1.5rem`. The words follow the app that shows the document, as chapter 3's do.

Any block takes a size:

| Param | Values |
| --- | --- |
| `width`, `min-width`, `max-width` | a length, a percentage of the box around it, or `auto` |
| `height` | a length, a percentage, or `auto` |
| `aspect-ratio` | width over height, such as `16/9`, or `none` |

A size includes the block's padding. A `width` on an item of a row makes it that wide instead of as wide as its content: `width=30%` in a row gives three to a line.

## A table too wide for a phone

A table with many columns is wider than a phone. `scroll` lets it scroll sideways inside its own box, so the rest of the page stays put:

```md
::: box {scroll}

| Plot | Gardener | 2023 | 2024 | 2025 | 2026 | Next year |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Maya Okafor | beans | cabbages | roots | potatoes | beans |
| 2 | Tomás Reyes | cabbages | roots | potatoes | beans | cabbages |

::: /box
```

## `box`, closer

`stack`, `row` and `grid` are boxes with their layout set: a stack is a box with `display=flex flex-direction=column`, a row one with `display=flex flex-direction=row flex-wrap=wrap`, a grid one with `display=grid`. You can write those on a `box` yourself, and the names come from CSS, so they mean what they mean there. The three names say what you mean in one word, and are what this book uses.

A box with none of them is `display=block`: its blocks come one under another, as on the page, and it is the box you put around anything to give it padding, a border or a background.

Next: [widths and breakpoints](10-widths-and-breakpoints.md), for layouts that change with the width: one column on a phone, two on a wider page.
