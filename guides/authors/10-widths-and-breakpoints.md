# 10. Widths and breakpoints

A newsletter is read on phones as well as on wide screens, and a layout that suits one rarely suits the other. This chapter makes the newsletter change with the width: the news and the work day in one column on a phone and side by side on a wider page, more padding where there is room, and the plots as a short list on a phone and a table on a wide page.

## The newsletter, at every width

```md snippet=breakpoints/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
news:
  - The water goes off on November 15. Empty your cans and hoses before then.
  - The seed swap is on December 5, in the library's back room.
  - Tomás has spare kale seedlings, free to a good bed.
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: grid {gap=m md:grid-template-columns="2fr 1fr" md:gap=l}

::: box

## News

::: each {in=(md.doc.news) as=item join=list}

- {{item}}

::: /each

::: /box

::: box {padding=m md:padding=l border-radius=m background=subtle}

## Work day

Saturday, November 7, from nine to noon. Bring gloves; we bring the tea.

::: /box

::: /grid

## The plots

::: box {hide=md}

- **Plot 1**: Maya Okafor, garlic.
- **Plot 2**: Tomás Reyes, kale.
- **Plot 4**: Priya Nair, leeks.

Plots 3 and 5 are free.

::: /box

::: box {show=md}

| Plot | Gardener | Growing | Since | Next year |
| ---: | --- | --- | --- | --- |
| 1 | Maya Okafor | garlic | 2019 | beans |
| 2 | Tomás Reyes | kale | 2023 | roots |
| 3 | free | | | |
| 4 | Priya Nair | leeks | 2021 | potatoes |
| 5 | free | | | |

::: /box
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/breakpoints) in the playground. With the page narrow, the work day's box comes under the news, and the plots are a list. Drag the bar between the text and the page until the page is wider than 768 pixels (folding the text away is quickest): the work day moves beside the news, a third of the width, its padding grows, and the list gives way to the table.

## Breakpoints

A *breakpoint* is a width at which a param takes another value. There are three, always the same:

| Breakpoint | From |
| --- | --- |
| `sm` | 480 pixels, a large phone |
| `md` | 768 pixels, a tablet |
| `lg` | 1024 pixels, a laptop |

A param with a breakpoint's name and a colon in front applies from that width up: `md:padding=l` is `l` from 768 pixels up. The same param without one is the value below it. So you write the layout for a phone first, and then what changes on wider pages:

```md
::: box {padding=s sm:padding=m lg:padding=l border=subtle}

Padding `s` on a small phone, `m` from 480 pixels, and `l` from 1024. Between 768 and 1024 it stays `m`: each value holds until a wider one takes over.

::: /box
```

In the newsletter, the grid has one column, as chapter 9's grid without columns does, until `md:grid-template-columns="2fr 1fr"` gives it two from `md` up. Its `gap` is `m`, and `l` from `md` up.

Most params take a breakpoint: the layout params of chapter 9, the common params of chapter 3, and those of chapter 11. A few don't, since they make no sense at one width and not another: `grid-area`, `scroll`, `fill`, `show` and `hide`, and `class`, `id`, `lang` and `dir`. A breakpoint on one of them is a diagnostic (`F008`), and so is a name other than `sm`, `md` and `lg`, such as `xl:gap=l`; the param is then left out.

## Showing and hiding

`show` and `hide` take a breakpoint as their value:

| Written | The block shows |
| --- | --- |
| `show=md` | from `md` up |
| `hide=md` | below `md` |
| `show=sm hide=lg` | from `sm` up to `lg` |

The newsletter writes the plots twice, a list for phones (`hide=md`) and a table for wider pages (`show=md`), so the reader sees one of them at any width. Keep both saying the same thing: which one a reader sees depends only on the width.

A hidden block is still on the page, only out of sight. A screen reader skips it too, so it doesn't read the plots twice. A plain Markdown viewer, such as GitHub's, shows both, one after the other.

## Which width

Breakpoints follow the width of the box a block is in, not the width of the window. At the top of a document that is the page itself, which is as wide as the app makes it: the whole window, a column beside a list, the playground's page pane. A document shown in a narrow column lays out as on a phone, however wide the window is.

Inside a box, a block's breakpoints follow that box. The same two-column grid, in a box at most 24rem wide, never gets its second column, since 24rem is less than 768 pixels:

```md
::: box {max-width=24rem}

::: grid {gap=m md:grid-template-columns="1fr 1fr"}

**News**

**Work day**

::: /grid

::: /box
```

So a part of a page lays out by the room it has, wherever you put it. A card in a grid follows the grid, not its own width: a card with `md:padding=l`, in a grid 1000 pixels wide, has `l`, though the card itself is only 300 pixels wide.

`each` and `if` aren't boxes, and neither are lists, quotes and table cells: a block in them follows the box around them.

## A box that needs a width

A box whose blocks use breakpoints has its width from where it is, never from its content, since its content waits for its width. Where a box would be as wide as its content, as in a row (chapter 9), it would then shrink to nothing. So such a box needs a `width`, and without one it is a diagnostic (`W114`). This one, in a row, has the grid's breakpoints in it and no width:

```text
::: row {gap=m}

::: box
::: grid {md:grid-template-columns="1fr 1fr"}
…
```

Give it one, and the grid in it follows that width:

```md
::: row {gap=m}

::: box {width=60%}

::: grid {gap=s md:grid-template-columns="1fr 1fr"}

**News**

**Work day**

::: /grid

::: /box

**Contacts**

::: /row
```

The same holds in a stack where the box isn't stretched across it (an `align-items` or `align-self` other than `stretch`), in a grid where the box has a `justify-self`, and for a box placed by position (chapter 11) without both an `inset-inline-start` and an `inset-inline-end`. A box at the top, or in a plain box, stack or grid, is as wide as what holds it, and needs nothing.

Next: [placing blocks](11-placing-blocks.md), for blocks in cells of a grid with names, at a place in a box, or kept in view as the page scrolls.
