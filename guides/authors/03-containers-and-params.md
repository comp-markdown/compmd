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
| `#greeting` | the component's id, for a link to it: `[the greeting](#greeting)` |
| `.notice` | a class, which a stylesheet of the app showing the document can style |
| `md:padding=l` | from a width up (chapter 10) |

Which params a component takes, and what values each takes, is up to the component. A param it doesn't know (`F008`), or a value it doesn't take (`F009`), is a diagnostic that says what it does take, and often offers to fix it: write `border=subtel`, and the playground offers `subtle`.

`lang` and `dir` are taken by every component, for a passage in another language: `lang=es` above tells the reader's browser, a screen reader and a spell checker that the greeting is Spanish. `dir=rtl` writes a passage right to left, as Arabic or Hebrew is, and `dir=auto` lets the text decide, for text whose language you don't know.

## The common params

Every block, whatever its component, has a box around it, and every component takes the same params for that box. The ones you'll use most:

| Param | Values |
| --- | --- |
| `padding` | `none`, `s`, `m`, `l`, or a length such as `1rem`; two of them for top and bottom, then the sides: `padding="s m"` |
| `border` | `none`, `subtle`, `strong`, or a CSS border such as `"1px dashed"` |
| `border-radius` | `none`, `s`, `m`, `l`, `full` for a pill, or a length |
| `background` | `none`, `subtle`, `strong`, `accent`, or a color such as `#e9f3ff` |
| `width`, `max-width` | a length or a percentage: `max-width=32rem` |

The words (`s`, `subtle`, `accent`) follow the look of the app that shows the document, light or dark, so a document written with them fits wherever it is shown. A color or a length is the same everywhere: use them when that's what you mean.

Part III adds the params that lay blocks out, side by side or in a grid.

## Long param lists

When the params don't fit on a line, continue them on the next line, which starts with `::: +`, directly under the opener:

```md
::: box {#greeting lang=es padding="s m"}
::: + border-radius=m background=subtle max-width=32rem

¡Hola, vecinos!

::: /box
```

A continuation line goes directly under the opener, a leaf or another continuation line, with no blank line between them.

## Slots

Some containers have parts with names, called *slots*, each started by a line of two colons and a `#`: `:: #else`. You'll use them with `if` and `each` in chapter 5, and give your own components slots in Part IV.

Next: [values and expressions](04-values-and-expressions.md), for text that comes from the frontmatter or is computed.
