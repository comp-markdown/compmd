---
title: A tour of CompMD
updated: "2026-10-29"
tools: [{name: a text editor}, {name: this page}]
md:
  version: 1
  syntax: [wiki-links]
---

# {{md.doc.title}}

CompMD is CommonMark and GFM with components on top, so everything plain Markdown does still works: **bold**, *italic*, ~~struck~~, `code`, [links](https://commonmark.org), autolinks such as https://github.github.com/gfm/, and footnotes.[^plain] The Plain Markdown tab shows what a renderer that knows nothing of CompMD makes of this page: it stays readable.

[^plain]: Every component has a fallback, the text a plain renderer shows in its place.

## Blocks

- Lists, tight or loose
- [x] Task lists
- [ ] with their boxes

1. Ordered lists
2. that count

> A quote.

> [!TIP]
> GFM's alerts come in five kinds: note, tip, important, warning and caution.

| You write | You get |
| --- | --- |
| `::: name` … `::: /name` | a container, with a body |
| `:: name[text]` | a leaf, one line |
| `:name[text]` | an inline component, inside a line of text |

Math inline, $e^{i\pi} + 1 = 0$, and as a block:

$$
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
$$

Wiki links, which this document turns on in its frontmatter: [[Getting started]], or [[Getting started|with a label]].

## Components

A container holds a body and closes by its name. Its params come in braces after the name:

::: box {padding=m border=subtle border-radius=m background=subtle}
Anything Markdown holds goes in a container: *text*, lists, other components.
::: /box

A leaf is one line. The text in brackets is its fallback:

:: pill[New in 1.0] {label="New in 1.0"}

An inline component sits in a line of text, :pill[like this]{label="like this" tone=accent}, and its fallback is part of the sentence.

```compmd component=pill {padding="2px 8px" border-radius=full background=(tone == "accent" ? "accent" : "subtle")}
params:
  label: string
  tone: plain | accent = plain
---
{{label}}
```

That fence defines `pill`. Definitions can sit anywhere in a document: the Components tab lists what this one defines and uses.

## Params

Params are written as in HTML attributes, with more kinds of values:

- `label="New"`, a quoted string, with interpolations: `title="Step {{n}}"`
- `gap=m`, a bare word
- `tone=(done ? "accent" : "plain")`, an expression in parentheses
- `cols=["a", "b"]`, JSON
- `#intro` and `.lead`, an id and a class
- `md:gap=l`, a param from a width up: `sm`, `md` or `lg`

A long param list goes on to the next line with `::: +`:

::: row {#params .demo gap=s align-items=center}
::: + md:gap=l justify-content=between
:pill[one]{label="one"} :pill[two]{label="two" tone=accent} :pill[three]{label="three"}
::: /row

## Values

`{{…}}` inserts a value: this page is "{{md.doc.title}}", updated {{md.doc.updated}}. Formatted values come from `value`, with its fallback written in: updated :value[October 29, 2026]{of=(md.date(md.doc.updated, "long"))}, with :value[a text editor and this page]{of=(md.list(md.doc.tools))}.

Write `\{{` for braces that aren't a value: \{{like this}}.

## Slots and control flow

`if` shows its body or its `else` slot. A slot starts with `:: #name` at the container's level:

::: if {when=(size(md.doc.tools) > 1)}
You need :value[2]{of=(string(size(md.doc.tools)))} tools.

:: #else
One tool is enough.
::: /if

`each` repeats its body once per item:

::: each {in=(md.doc.tools) as=t join=list}
- {{t.name}}
::: /each

The next documents in the menu go further: definitions and slots, expressions, data, inputs, layout and components in other languages.
