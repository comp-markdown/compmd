---
title: Learn CompMD
author: {name: Ada Byron, role: editor}
released: "2026-10-29"
price: 12
tags: [markdown, components, data]
reading: {title: Piranesi, page: 120, pages: 272}
md:
  version: 1
  syntax: [wiki-links]
  data:
    - from: ./books.csv
      as: books
      schema: {title: string, author: string, year: number, pages: number, read: boolean}
  inputs:
    since: number = 1960
    unread: boolean = false
---

# {{md.doc.title}}

CompMD is Markdown with components: boxes and cards, values from data, lists that repeat, inputs the reader changes, layouts that follow the width of the page. This tutorial takes them one at a time. The text on the left is the document; the page on the right is what it renders, as you type.

Each lesson ends with **Try it**: a change to make in the text, and what to watch for on the page. Nothing you do here is saved, and the menu at the top brings the original back.

## 1. A document

A CompMD document is a Markdown file that starts with *frontmatter*: YAML between two `---` lines. CompMD's own settings go under the `md` key. Every other key is yours, and the document reads it as `md.doc.<key>`.

The heading above is written `# {{md.doc.title}}`: the `title` key, inserted where `{{…}}` stands. This document is by {{md.doc.author}}, the `author` key, which is a record: a record with a `name` reads as its name.

> [!TIP]
> **Try it.** Change `title: Learn CompMD` in the frontmatter, and watch the heading follow. Then open the **Data** tab beside the page: it shows every key the frontmatter gives the page.

## 2. The Markdown you know

Everything Markdown does still works, as CommonMark and GitHub write it: **bold**, *italic*, ~~struck~~, `code`, [links](https://commonmark.org), autolinks such as https://commonmark.org, and footnotes.[^plain]

[^plain]: Footnotes gather at the end of the page.

- Lists, tight or loose
- [x] Task lists
- [ ] with their boxes

| Tables | with | columns |
| --- | :---: | ---: |
| left | center | right |

> [!NOTE]
> Alerts come in five kinds: note, tip, important, warning and caution.

Math is written in TeX, inline as $e^{i\pi} + 1 = 0$, or as a block. This page shows the TeX as code; a host with a math renderer typesets it:

$$
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
$$

Wiki links are an option a document turns on, as this one does with `syntax: [wiki-links]`: [[Getting started]], or [[Getting started|with a label]]. Where a wiki link leads is the host's to say, so here they stay text.

> [!TIP]
> **Try it.** Open the **Plain Markdown** tab. It shows this page as a renderer that knows nothing of CompMD shows it: every component becomes readable text, so a CompMD file still reads well on GitHub or in any Markdown viewer.

## 3. Containers

A *container* wraps a body in a component. It opens with three colons and the component's name, and closes with the same name after a slash:

::: box {padding=m border=subtle border-radius=m}
This is a `box`, the simplest component: a box around its body. Anything Markdown holds goes in it: *text*, lists, other containers.

::: box {padding=s background=subtle border-radius=s}
A box inside a box. Each closer names its container, so it's always clear which one ends.
::: /box
::: /box

Every component line stands on a line of its own, with a blank line before and after it.

> [!TIP]
> **Try it.** Delete the last `::: /box` line. The page still renders, and a diagnostic under the text says which container was never closed, and clicking its place selects the line it means. Put the closer back, or press undo.

## 4. Params

What's in the braces are the component's *params*. They read like HTML attributes, with more kinds of values:

| You write | It is |
| --- | --- |
| `title="Setup"` | text, in quotes; `title="Step {{n}}"` inserts a value |
| `gap=m` | a bare word or number |
| `open` | a flag: `true` |
| `count=(size(rows))` | an expression, in parentheses (lesson 6) |
| `cols=["a", "b"]` | JSON |
| `.wide`, `#intro` | a class, an id |
| `md:gap=l` | a param from a width up (lesson 12) |

Every block component takes the same *common params* for its box: `padding`, `border`, `border-radius`, `background`, `width`, `max-width` and more. Their values are tokens (`none`, `s`, `m`, `l` for sizes; `subtle`, `strong`, `accent` for colors) or CSS values such as `1rem` or `#e9f3ff`:

::: box {#params .demo padding=l border=strong border-radius=l background=accent max-width=28rem}
A box with an id, a class, a large padding, a strong border, large corners, the accent background, and a width of at most 28rem.
::: /box

A long param list goes on to the next line, which starts `::: +`:

::: box {padding=m border=subtle}
::: + border-radius=full background=subtle
A pill.
::: /box

> [!TIP]
> **Try it.** Change `border=strong` to `border=strng`. The box loses its border, and the diagnostic says what `border` takes, with a button that writes `strong` back.

## 5. Values

`{{path}}` inserts a value from the frontmatter, the data or the inputs: this page was released on {{md.doc.released}} and costs {{md.doc.price}} dollars. A path is names and dots, nothing more.

Anything computed or formatted comes from the inline component `value`, whose `of` param is an expression. What's in its brackets is its *fallback*, the text a plain renderer shows instead, so write what it usually says:

- Released :value[October 29, 2026]{of=(md.date(md.doc.released, "long"))}.
- Costs :value[$12.00]{of=(md.currency(md.doc.price, "USD"))}, or :value[$120.00]{of=(md.currency(md.doc.price * 10, "USD"))} for ten.
- Tagged :value[markdown, components, and data]{of=(md.list(md.doc.tags))}.
- :value[3 tags]{of=(string(size(md.doc.tags)) + " " + md.plural(size(md.doc.tags), "tag", "tags"))}.

To write braces that aren't a value, start them with a backslash: \{{like this}}.

> [!TIP]
> **Try it.** Change `price: 12` to `price: 9.5`, and watch every price follow. Then change `"long"` to `"full"` in the date.

## 6. Expressions

An expression in parentheses can compute any param. Expressions are [CEL](https://cel.dev), a small language with no loops and no side effects: `==`, `&&`, `? :`, `size(…)`, and methods on lists such as `filter`, `map` and `exists`.

::: box {padding=m border-radius=m background=(md.doc.price > 10 ? "accent" : "subtle")}
This box has the accent background while the price is over 10, and the subtle one otherwise: `background=(md.doc.price > 10 ? "accent" : "subtle")`.
::: /box

On a list, an expression counts, filters and adds: the reading list has :value[6 books]{of=(string(size(md.data.books)) + " books")}, :value[3 of them read]{of=(string(size(md.data.books.filter(b, b.read))) + " of them read")}, :value[1,719 pages]{of=(md.number(md.sum(md.data.books.map(b, b.pages)), 0) + " pages")} in all.

> [!TIP]
> **Try it.** Change `price: 12` to `price: 8`: the box above turns subtle. An expression with a mistake, say `md.doc.prise`, gives a diagnostic that points into the expression.

## 7. Showing and repeating

Two built-in containers decide what shows. `if` shows its body when `when` is true, and its `else` slot otherwise. A *slot* starts with a `:: #name` line:

::: if {when=(md.data.books.exists(b, !b.read))}
There are books left to read.

:: #else
Every book is read.
::: /if

`each` repeats its body once per item of a list, with the item under the name `as` gives it. With `join=list`, the items make one list; `before` shows above it when there are items, and `empty` when there are none:

::: each {in=(md.sortBy(md.data.books, "year")) as=b join=list}
1. *{{b.title}}*, {{b.author}} ({{b.year}})

:: #before
**The reading list, oldest first:**

:: #empty
The reading list is empty.
::: /each

The ordered list numbers on from one item to the next, and inside `each`, `md.loop` knows where it is: `md.loop.number`, `md.loop.first`, `md.loop.last`. With `join=inline`, the items make one paragraph instead, joined by `sep` and, before the last, `last`. Here, the authors:

::: each {in=(md.unique(md.data.books.map(b, b.author))) as=a join=inline sep=", " last=" and "}
{{a}}
::: /each

> [!TIP]
> **Try it.** Change `md.sortBy(md.data.books, "year")` to `md.data.books.filter(b, b.pages > 300)`: only the two books over 300 pages stay, numbered 1 and 2.

## 8. Your own components

A component of your own is a fence: `compmd component=<name>`, the params that style its box, then a *signature* (its params with their types, and its slots), a `---` line, and the body. A definition doesn't show on the page, so look for `note` in the text on the left.

```compmd component=note {padding=m border=subtle border-radius=m background=(kind == "warning" ? "subtle" : "none")}
params:
  kind: tip | warning = tip
  title: string?
slots: [default, footer]
---
**{{title}}**

{{md.slot}}

{{md.slot.footer}}
```

In the body, `{{title}}` is the param, `{{md.slot}}` the body the caller writes, and `{{md.slot.footer}}` the `footer` slot. Defined anywhere in the document, it's used anywhere:

::: note {title="Before you start" kind=warning}
Back up your settings.

:: #footer
*It takes a minute.*
::: /note

Without a body, it's a *leaf*: two colons, and a fallback in brackets.

:: note[A note without a body] {title="A leaf"}

A component whose body is one paragraph can also be *inline*, inside a sentence, as `tag`, defined in the text, is:

```compmd component=tag {padding="1px 8px" border-radius=full background=subtle}
params:
  label: string
---
{{label}}
```

This sentence has a :tag[draft]{label="draft"} in it, and a :tag[second one]{label="second one"}.

> [!TIP]
> **Try it.** Add `danger` to `kind: tip | warning = tip`, then write `kind=danger` on the note. Write `kind=dangr` instead: the diagnostic lists the kinds the note takes. The **Components** tab lists every component this document defines and uses.

## 9. Templates

A component can repeat a template itself: a *template slot* renders once per element of a list param, under a name the caller picks. Here `ranked`, defined in the text, numbers whatever it's given:

```compmd component=ranked {display=flex flex-direction=column gap=s}
params:
  items: array
  as: name
slots:
  default: {each: items, as: as}
---
{{md.slot}}
```

::: ranked {items=(md.sortBy(md.data.books, "pages")) as=b}
**{{md.loop.number}}.** {{b.title}}, {{b.pages}} pages
::: /ranked

The caller decides what each element looks like; the component decides where it goes.

> [!TIP]
> **Try it.** Change `"pages"` to `"title"`, or `as=b` to `as=book`, and the template's `b.` to `book.`.

## 10. Data

Data comes in through the frontmatter's `data` key: a file, a URL or a source the host provides, read as `md.data.<as>`. This document reads `books.csv`, with a `schema` that types its columns, so `year` and `pages` are numbers and `read` is true or false. The tab above the text shows the file, read-only.

::: each {in=(md.data.books.filter(b, b.read)) as=b join=list}
- *{{b.title}}*: :value[304 pages]{of=(md.number(b.pages, 0) + " pages")}
::: /each

Read so far: :value[840 pages]{of=(md.number(md.sum(md.data.books.filter(b, b.read).map(b, b.pages)), 0) + " pages")}.

> [!TIP]
> **Try it.** Open `books.csv` in the tabs above the text, then come back to `tour.comp.md` and change `b.read` to `!b.read`.

## 11. Inputs

*Inputs* are values the reader changes. The frontmatter declares each with a type and a default, the page reads it as `md.input.<name>`, and an input component edits it:

:: number-input[Published since: 1960] {bind=since}

:: toggle-input[Unread only: no] {bind=unread}

::: each {in=(md.data.books.filter(b, b.year >= md.input.since && (!md.input.unread || !b.read))) as=b join=list}
- *{{b.title}}* ({{b.year}})

:: #empty
No book fits.
::: /each

The same inputs are in the bar above the tabs. There are five kinds: `date-input`, `number-input`, `text-input`, `select-input` for a list of choices, and `toggle-input`. **Inputs: plan a stay**, in the menu, uses them all.

> [!TIP]
> **Try it.** Set the year to 2000, or turn on *unread only*, and watch the list follow.

## 12. Layout

Three built-ins lay out their blocks: `stack` one below the other, `row` side by side, `grid` in columns. Each top-level block inside is one item.

::: grid {min=9rem gap=s md:gap=m}
::: each {in=(md.data.books) as=b}
::: box {padding=s border=subtle border-radius=m}
**{{b.title}}**

{{b.author}}, {{b.year}}
::: /box
::: /each
::: /grid

`min=9rem` fits as many columns as there's room for, at least 9rem wide each. `md:gap=m` widens the gaps from the `md` width up. The widths are `sm` (480px), `md` (768px) and `lg` (1024px), measured on the box around the block, not the window.

::: box {padding=s background=subtle border-radius=s show=md}
You see this box when the page is at least 768px wide.
::: /box

::: box {padding=s background=subtle border-radius=s hide=md}
You see this box when the page is narrower than 768px.
::: /box

> [!TIP]
> **Try it.** Drag the handle between the text and the page to make the page narrow: the grid drops columns, and the two boxes above trade places.

## 13. Components in JavaScript

When CompMD isn't enough, a component can be code: JSX or TSX for React, Svelte, or plain JavaScript. The page runs it in a sandbox, with no network. In the text, a `jsx` fence holds `meter`'s code, and a signature fence types its params.

```compmd component=meter signature
params:
  value: number
  max: number
```

```jsx component=meter runtime=react@19
export default function Meter({ value, max, compmd }) {
  const share = Math.max(0, Math.min(1, value / max));
  return (
    <svg role="img" aria-label={compmd.fallback.text} viewBox="0 0 100 6" width="100%" height="10" preserveAspectRatio="none">
      <rect width="100" height="6" rx="3" fill="currentColor" opacity="0.15" />
      <rect width={share * 100} height="6" rx="3" fill="currentColor" />
    </svg>
  );
}
```

Reading *{{md.doc.reading.title}}*, page {{md.doc.reading.page}} of {{md.doc.reading.pages}}:

:: meter[Page 120 of 272] {value=(md.doc.reading.page) max=(md.doc.reading.pages)}

> [!TIP]
> **Try it.** Change `page: 120` in the frontmatter to `page: 250`. Then change the code: `rx="3"` to `rx="0"` gives the bar square ends, and the page shows the new code as you type.

## 14. Where next

The other documents in the menu go further, each on one side of CompMD:

- **Definitions and slots**, and **Expressions: a changelog**, for components built from components.
- **Data: a sales dashboard**, with charts in React and Svelte.
- **Inputs: plan a stay**, with every kind of input.
- **Layout: Kyoto in four days**, with grid areas and blocks placed by position.
- **Diagnostics and their fixes**, five mistakes and how the editor mends them.

The [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) says everything precisely.
