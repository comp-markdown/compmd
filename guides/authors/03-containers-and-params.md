# 3. Containers and params

Components turn a text into a page: a notice that stands out, a box inside a box, a passage in another language. This chapter is about how you write any component, and the params that set how it looks. The newsletter gets a notice, a work day with its list, and a greeting from the neighbors.

## The newsletter, with boxes

```md snippet=containers/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: box {padding=m border-radius=m background=accent}

**The water goes off on November 15** for the winter. Empty your cans and hoses before then.

::: /box

## Work day

::: box {padding=m border=subtle border-radius=m}

Saturday, November 7, from nine to noon. Bring gloves; we bring the tea.

::: box {padding=s border-radius=s background=subtle}

**On the list:** turn the compost, mend the gate by plot 4, plant the garlic.

::: /box

::: /box

## From our neighbors

::: box {#greeting lang=es padding="s m" border-radius=m background=subtle}
::: + max-width=32rem

*¡Hola, vecinos!* La huerta de la calle Olmo os invita a la jornada del sábado.

::: /box
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/containers) in the playground.

## Three ways to write a component

A component is written in one of three ways, and the number of colons tells which:

| Form | Written | For |
| --- | --- | --- |
| Container | `::: box {…}`, a body, then `::: /box` | a component around content you write |
| Leaf | `:: chart[Weekly sign-ups] {…}`, one line | a component that makes its content itself |
| Inline | `:value[12 plots]{…}`, inside a sentence | a component in the middle of text |

Each has a name, and params in braces, which may be left out. A leaf and an inline component also have a *fallback* in brackets, which the next sections explain.

The components you can use come from three places: those built into CompMD, such as `box`; those from *packs* other people publish (chapter 16); and your own, written in CompMD itself (Part IV). Built in are `box` and the layouts `stack`, `row` and `grid` (Part III), `if` and `each` (chapter 5), `value` (chapter 4), and the inputs a reader changes (chapter 7).

## Containers

A container opens with three colons and its name, and closes with three colons, a slash and the same name. Its body is any Markdown: paragraphs, lists, tables, and other containers.

Containers nest, as the work day's list nests in the work day. Because every closer names its container, it's always clear which one a line ends, even with boxes in boxes. A closer with a misspelt name, such as `::: /bxo`, is a diagnostic (`F003`) that names the container still open; a container never closed is `F004`, and it runs to the end of what holds it.

A container can sit in a list item or a quote, indented like the item's text:

```md
- Sign up by November 30.

  ::: box {padding=s border=subtle}

  Plots go to last year's gardeners first.

  ::: /box
```

In a quote, each line of the container starts with `>`, as the quote's other lines do:

```md
> ::: box {padding=s border=subtle}
>
> The early bird gets the sunniest plot.
>
> ::: /box
```

A container opens and closes in the same list item or quote. A closer in another one (`F032`) is ignored, and the container closes where its own list item or quote ends.

A container may be empty, its closer right under its opener, and may hold another of the same name.

## Fallbacks

A leaf or an inline component has no body you write: it makes its content itself, such as a chart or a computed number. What's in its brackets is its *fallback*: the text shown where the component can't be, by a plain renderer, or when something goes wrong. Write in it what the component usually shows, so the text reads well on its own: `:: chart[Sign-ups rose from 12 to 31 this year] {…}`.

A container's fallback is its body, as Markdown, so a plain renderer shows everything you wrote in it.

## Params

Params go in braces after the name, separated by spaces. They read like a list of settings:

| Written | Is |
| --- | --- |
| `padding=m` | a word or a number, as written |
| `padding="s m"` | text in quotes, which may hold spaces |
| `scroll` | a setting turned on |
| `background=(…)` | an expression, computed (chapter 4) |
| `tags=["kale","leeks"]` | a list, written as in JSON, for a component that takes one |
| `#greeting` | the component's id, for a link to it: `[the greeting](#greeting)` |
| `.notice` | a class, which a stylesheet of the app showing the document can style |
| `md:padding=l` | from a width up (chapter 10) |

A few rules about how values are written:

- A value without quotes ends at the next space or `}`, and can't hold parentheses or braces: `title={{md.doc.title}}` is a mistake (`F011`), and `title="{{md.doc.title}}"` is what you mean. A value with nothing after the `=` is one too; `title=""` is empty text.
- In quotes, a backslash writes a quote or a backslash: `"Say \"hi\""`.
- Quotes always make text. Without them, the component decides: `count=3` is the number 3 to a component that takes a number, and `count="3"` is the text *3*.
- Each param is given once, continuation lines included (below); the second time is a diagnostic (`F017`), and the first one counts.

Which params a component takes, and what values each takes, is up to the component. A param it doesn't know (`F008`), or a value it doesn't take (`F009`), is a diagnostic that says what it does take, and often offers to fix it: write `border=subtel`, and the playground offers `subtle`.

`lang` and `dir` are taken by every component, for a passage in another language: `lang=es` above tells the reader's browser, a screen reader and a spell checker that the greeting is Spanish. `dir=rtl` writes a passage right to left, as Arabic or Hebrew is, and `dir=auto` lets the text decide, for text whose language you don't know.

## The common params

Every block, whatever its component, has a box around it, and every component takes the same params for that box. These set its look and its size:

| Param | Values |
| --- | --- |
| `padding` | `none`, `s`, `m`, `l`, or a length such as `1rem`; two of them for top and bottom, then the sides: `padding="s m"` |
| `padding-block`, `padding-inline` | the same, for top and bottom only, or for the sides only; two of them for one end, then the other |
| `padding-block-start`, `padding-block-end`, `padding-inline-start`, `padding-inline-end` | one of them, for one side: the top, the bottom, the side the text starts from, the side it ends at |
| `border` | `none`, `subtle`, `strong`, or a CSS border such as `"1px dashed"` |
| `border-radius` | `none`, `s`, `m`, `l`, `full` for a pill, or a length |
| `background` | `none`, `subtle`, `strong`, `accent`, or a color such as `#e9f3ff` |
| `width`, `max-width`, `min-width`, `height` | a length, a percentage, or `auto`: `max-width=32rem` |
| `aspect-ratio` | width over height, such as `16/9`, or `none` |

A length is a number with `px`, `rem`, `em` or `%`; `1rem` is the page's text size. A width or height counts the padding in it.

The words (`s`, `subtle`, `accent`) follow the look of the app that shows the document, light or dark, so a document written with them fits wherever it is shown. Unless the app changes them, `s`, `m` and `l` are half the text's size, the text's size and twice it (0.5, 1 and 2 rem) for padding, and 4, 8 and 16 pixels for corners; `subtle` and `strong` are faint and stronger tints of the text's own color. A color or a length is the same everywhere: use them when that's what you mean.

Sides are named `start` and `end`, not left and right, so a passage written right to left (`dir=rtl`) gets its padding on the right sides too.

The rest of the common params place a block among others: `grid-area`, `order`, `align-self`, `justify-self`, `show`, `hide`, `inset-…`, `scroll`, `fill` and `sticky`. They are Part III's, in [chapters 9](09-boxes-rows-stacks-and-grids.md), [10](10-widths-and-breakpoints.md) and [11](11-placing-blocks.md). These are all the styling a document can write: there is no param for raw CSS, so a name outside the list is `F008`, and a value outside a param's values is `F009`.

## Ids and classes

An id names one block, so a link can lead to it. Write it with `#` in the braces, `{#greeting}`, or as `id=greeting`, but not both (`F017`). `[the greeting](#greeting)` then scrolls to the block.

A class is a name for a kind of block, which a stylesheet of the app showing the document can style: `{.notice}`. A class does nothing by itself; the app has to know it. A block can have several, `{.notice .wide}`, and `class=(…)` adds one that is computed: `{.plot class=("crop-" + p.crop)}` gives the classes *plot* and *crop-garlic* (chapter 5 has `p`). Classes add up in the order written.

Ids and classes are names: a letter first, then letters, digits, `-` or `_`.

## Long param lists

When the params don't fit on a line, continue them on the next line, which starts with `::: +`, directly under the opener:

```md
::: box {#greeting lang=es padding="s m"}
::: + border-radius=m background=subtle max-width=32rem

¡Hola, vecinos!

::: /box
```

A continuation line goes directly under the opener, a leaf or another continuation line, with no blank line between them. Anywhere else, it is a diagnostic (`F012`), and is ignored.

## When a component line is wrong

Each mistake in a component line is one diagnostic, and the page keeps as much as it can:

| You wrote | Code | The page |
| --- | --- | --- |
| `:::box`, without the space | `F016` | shows the line as text |
| `::: /bxo`, a misspelt closer | `F003` | closes the innermost open container |
| a container with no closer | `F004` | runs it to the end of what holds it |
| `::: /box` with no box open | `F005` | ignores the line |
| `padding=xl`, a value the param doesn't take | `F009` | leaves the param out |
| `title=Hi`, a param the component doesn't take | `F008` | leaves the param out |
| `padding=m padding=s` | `F017` | uses the first |
| `title={{md.doc.title}}`, braces without quotes | `F011` | keeps the params before the mistake |
| `:: #else` in a container without that slot | `F006` | adds what follows to the body |
| a container line right under a paragraph's last line | `W108` | shows it as intended; a plain renderer may not, so add a blank line |

## Slots

Some containers have parts with names, called *slots*, each started by a line of two colons and a `#`: `:: #else`. You'll use them with `if` and `each` in chapter 5, and give your own components slots in Part IV.

Next: [values and expressions](04-values-and-expressions.md), for text that comes from the frontmatter or is computed.
