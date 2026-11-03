# 11. Placing blocks

Chapters 9 and 10 let blocks flow into a layout, one after another. This chapter puts blocks in particular places: in cells of a grid with names, at a place in a box, as on a map, or kept in view while the page scrolls. The newsletter gets a map of the garden, with its plots where they are, and its notes beside the map on a wide page.

## The newsletter, with a map

```md snippet=placing/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic, x: "6%", y: "10%"}
  - {number: 2, gardener: Tomás Reyes, crop: kale, x: "38%", y: "10%"}
  - {number: 3, gardener: "", crop: "", x: "70%", y: "10%"}
  - {number: 4, gardener: Priya Nair, crop: leeks, x: "6%", y: "55%"}
  - {number: 5, gardener: "", crop: "", x: "38%", y: "55%"}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: grid {gap=m grid-template-areas="'map' 'plots' 'notes'" md:grid-template-columns="2fr 1fr" md:grid-template-areas="'map notes' 'plots notes'"}

::: box {grid-area=map aspect-ratio=3/2 border=subtle border-radius=m background=subtle}

::: each {in=(md.doc.plots) as=p}

::: box {inset-inline-start=(p.x) inset-block-start=(p.y) width=26% padding=s border-radius=s background=(p.gardener == "" ? "accent" : "strong")}

**{{p.number}}**

::: /box

::: /each

::: box {inset-inline-end=4% inset-block-end=8% padding=s border=strong border-radius=s}

Shed

::: /box

::: box {inset-inline-start=0 inset-block-end=0 padding=s}

Gate

::: /box

::: /box

::: stack {grid-area=plots gap=s}

## The plots

::: each {in=(md.doc.plots) as=p}

::: box {padding=s border=subtle border-radius=s}

::: if {when=(p.gardener != "")}

**Plot {{p.number}}**: {{p.gardener}}, growing {{p.crop}}.

:: #else

**Plot {{p.number}}** is free for next year.

::: /if

::: /box

::: /each

::: /stack

::: stack {grid-area=notes gap=s align-self=start md:sticky=top}

## Notes

::: box {padding=m border-radius=m background=accent}

**The water goes off on November 15.**

::: /box

::: box {padding=m border-radius=m background=subtle}

**Work day:** Saturday, November 7, from nine to noon. Mend the gate first.

::: /box

::: /stack

::: /grid
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/placing) in the playground. The map is a box three wide by two high, with the five plots on it in two rows, the free ones in the accent colour, the shed in its far corner and the gate at the bottom. With the page narrow, the map, the plots and the notes come one under another. From 768 pixels up (drag the bar between the text and the page, or fold the text away), the notes move beside the map and the plots, and stay in view as you scroll down the list of plots. Move a plot by changing its `x` or `y`.

## Cells with names

`grid-template-areas` draws a grid's cells as words, a row in quotes for each row of the grid, a name for each cell:

```text
grid-template-areas="'map notes' 'plots notes'"
```

This is two rows of two cells. The name `notes` is in both rows of the second column, so the area called `notes` takes both. `map` is the first row's first cell, `plots` the second row's. A block goes in an area with `grid-area`: `grid-area=map` puts the map box there. A `.` instead of a name is a cell left empty. Every row has as many names as the others; rows of different lengths are a diagnostic (`F009`). An area must also be a rectangle: the same name in cells that don't make one isn't reported, but the browser then ignores the grid's areas.

The columns' widths still come from `grid-template-columns` (here `2fr 1fr`), and the rows are as tall as their content, or what `grid-template-rows` says, written as columns are.

`grid-template-areas` takes breakpoints, and `grid-area` doesn't: a block always has the same name, and the grid says where each name goes at each width. So a grid that names its areas from `md` up names them below it too. In the newsletter, below `md` they are one column, `'map' 'plots' 'notes'`; from `md` up, two. Without areas at the narrowest width, the blocks' names would point at areas that aren't there, and the grid would lay them out in extra rows and columns of its own. Spell the names alike in both places: a name the grid doesn't have isn't a diagnostic.

## On screen, and in reading order

Areas move blocks on screen, but never in the text. A screen reader, the keyboard's tab order, a copy and paste, and a plain Markdown viewer all follow the order you wrote. So write blocks in the order they should be read, and lay them out from there. Where blocks hold links or inputs, keep the order on screen close to the written one, so the keyboard doesn't jump about.

The same holds for `order`, which moves one block to the start or the end of its row, stack or grid: `order=first`, `order=last`, or `order=none` for where it was written. Here *Plant the garlic* shows first, though it is written last:

```md
::: row {gap=s}

::: box {padding=s border=subtle}

Turn the compost

::: /box

::: box {padding=s border=subtle}

Mend the gate

::: /box

::: box {padding=s background=accent order=first}

Plant the garlic

::: /box

::: /row
```

## Lining one block up

A layout lines up all its items with `align-items` (chapter 9). One item can differ:

| Param | Values | Does |
| --- | --- | --- |
| `align-self` | `start`, `center`, `end`, `stretch` | in a row or a grid, where the block is from top to bottom; in a stack, from side to side |
| `justify-self` | `start`, `center`, `end` | in a grid, where the block is in its cell from side to side; in a row, `end` pushes the block, and the blocks after it, to the end |

In the newsletter, the notes are `align-self=start`, as tall as their content. A block in a grid is stretched to its area's height by default, and the notes' area is as tall as the map and the plots together.

`justify-self=end` in a row is the way to put a block at the end and the others at the start:

```md
::: row {gap=s align-items=center}

**Elm Street Garden**

November 2026

::: box {justify-self=end padding="s m" border-radius=full background=accent}

Sign up by November 30

::: /box

::: /row
```

## Kept in view

`sticky=top` keeps a block at the top of the page while the page scrolls, as long as what holds it is still in view; `sticky=bottom` keeps it at the bottom, and `sticky=none` lets it scroll. A block sticks within its own area: in the newsletter's grid, the notes stay in view while the reader scrolls past the plots, and leave with the grid.

The notes stick only from `md` up (`md:sticky=top`): below it they come after the plots, and there is nothing left to scroll past. They are `align-self=start` too, since a block as tall as its area has no room to move in it.

## At a place in a box

The four *insets* put a block at a place in the box around it, as a note is pinned on a board. The block is then out of the flow: the other blocks lay out as if it weren't there, and it is drawn over them.

| Param | Measured from the box's |
| --- | --- |
| `inset-inline-start` | start edge: the left, in a language written left to right |
| `inset-inline-end` | end edge: the right |
| `inset-block-start` | top |
| `inset-block-end` | bottom |

Each takes a length, a percentage of the box's width (for the inline ones) or height (for the block ones), or `auto`, the default, which leaves it unset. The names say *start* and *end* rather than left and right, so a page in a language written right to left, such as Arabic, mirrors as it should: chapter 3's `dir=rtl`.

In the newsletter, each plot's label is placed with `inset-inline-start=(p.x)` and `inset-block-start=(p.y)`, from the plot's record, and the shed from the box's end and bottom. A placed block is as wide as its content, or as its `width` says: each plot is 26% of the map. With both `inset-inline-start` and `inset-inline-end`, it stretches between them. Among placed blocks, the later ones are on top.

The box a block is placed in is the nearest box around it: a `box`, `stack`, `row` or `grid`, or a component's box. `each` and `if` aren't boxes, nor are lists, quotes and table cells, so the plots' labels are placed in the map, through the `each`.

### The box needs a height

Placed blocks give their box no height, since they are out of the flow. So a box that places blocks needs one: a `height`, an `aspect-ratio`, as the map has, or `fill` (below). Without one it is a diagnostic (`W119`):

```text
::: box {padding=m}
::: box {inset-inline-end=0 inset-block-start=0}
New!
::: /box
::: /box
```

A block placed outside any box, at the top of the document, is `W119` too: it is placed against the page, whose height is the whole document's.

### Placed from a width up

The insets take breakpoints. `md:inset-inline-start=60%` places a block from `md` up and leaves it in the flow below; `md:inset-inline-start=auto` takes a block placed below `md` back into the flow from there. The box then needs its height at every width where it places a block: `W119` is checked at each.

## Filling the height

`fill` makes a block as tall as the room its box leaves, so a list in an app's side panel reaches the panel's bottom. It matters only where the document is shown in an area of a fixed height. In a page that grows with its content, as the playground's does, it does nothing. Like `scroll` (chapter 9), it is written alone, and takes no breakpoint.

## Components without a box

Every component has a box, so a component in a grid is one item, as a box is. A component you write yourself (chapter 12) can instead have none, with `display=contents` on its definition: its blocks are then items of the layout it is in, one by one, as if you had written them there. Such a component takes no `padding`, `width` or other param for a box (`W120`), since it has none. `display=contents` goes only on a definition; on a box in a document, it is `F009`.

Next: [definitions](12-definitions.md), for components of your own, written in CompMD.
