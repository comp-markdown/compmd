# 15. Signatures and fallbacks

A component's signature is what every use of it relies on: the params it takes, of which types, which ones it needs, and its slots. This chapter moves the newsletter's components into a file of their own, a kit any issue can import, gives them complete signatures, and shows what a page shows when a component can't be drawn.

## The newsletter, with a kit

The kit is a CompMD document of its own, in a folder beside the newsletter:

````md snippet=signatures/parts/garden-kit.comp.md
---
md:
  version: 1
  export:
    names: [plot-card, notice]
garden: Elm Street Garden
---

# The garden's components

The components every issue of the {{md.doc.garden}} newsletter uses.

```compmd component=plot-card {padding=s border=subtle border-radius=m}
params:
  number: number
  gardener: string?
  crop: string = "something new"
  size: small | large = small
---
# Plot {{number}}, {{size}}

::: if {when=(gardener == null)}

Free for next year: ask at the {{md.doc.garden}} shed.

:: #else

**{{gardener}}**, growing {{crop}}.

::: /if
```

```compmd component=notice {padding=m border-radius=m background=(tone == "urgent" ? "accent" : "subtle")}
params:
  title: string
  tone:
    type: plain | urgent
    default: plain
    doc: An urgent notice has the accent background.
slots: [default]
---
# {{title}}

{{md.slot}}
```
````

The newsletter imports what it uses from it:

```md snippet=signatures/garden.comp.md
---
md:
  version: 1
  import:
    - from: ./parts/garden-kit.comp.md
      names: [plot-card, notice]
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: notice {title="The water goes off on November 15" tone=urgent}

Empty your cans and hoses before then.

::: /notice

## The plots

::: grid {min=12rem gap=m}

:: plot-card[Plot 1, large: Maya Okafor, growing garlic] {number=1 gardener="Maya Okafor" crop=garlic size=large}

:: plot-card[Plot 2, small: Tomás Reyes, growing something new] {number=2 gardener="Tomás Reyes"}

:: plot-card[Plot 3, small: free for next year] {number=3}

::: /grid
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/signatures) in the playground.

The page shows the notice with the accent background, then three cards: *Plot 1, large* for Maya's garlic, *Plot 2, small* for Tomás, growing *something new*, since the use gives no crop, and *Plot 3, small*, free for next year.

## A file of components

Any CompMD document can offer its components to others. It lists them in its frontmatter, under `md.export`:

```text
md:
  export:
    names: [plot-card, notice]
```

- **`names`** lists the components it offers. Those it doesn't list stay its own: a kit can have helpers its components use, which no other document sees.
- A name can be offered under another one, `{name: notice, as: garden-notice}`, and is then imported by that name.
- `names: "*"` offers every component the file defines.

The newsletter imports them under `md.import`, as it would a pack (chapter 16), with the file's path in `from`:

```text
md:
  import:
    - from: ./parts/garden-kit.comp.md
      names: [plot-card, notice]
```

A path starts with `./`, and is read from the folder of the document it's written in. The kit is in a folder of its own, `parts`, so the playground lists only the newsletter as the example's document, with the kit as a file it reads. Next month's issue imports the same kit, and every issue's cards change together when the kit does.

What can go wrong:

| Written | Diagnostic |
| --- | --- |
| a file that isn't there | `F026`, and the uses of its names show their fallbacks |
| a name the file doesn't export | `F021`, on that name |
| two files that import each other | `F020`, on the import that closes the circle |

A pack (chapter 16) is a file like the kit, published for anyone to import, with its own name and version under `md.export`.

## What a component sees, from a kit

A component reads the kit's frontmatter, not yours: `{{md.doc.garden}}` in `plot-card` is the kit's `garden`, wherever the card is used. So a kit means the same in every document that imports it.

Slot content is the other way round: it is written in your document, so it reads your frontmatter (chapter 13). The notice's message could say `{{md.doc.issue}}`, and it would be your issue.

## Which name wins

A document can name a component from several places. When two have the same name, this decides:

| Comes first | Where the name comes from |
| --- | --- |
| 1 | the document's own definitions |
| 2 | names an import lists |
| 3 | `"*"` imports, and the built-in components |
| 4 | components the app showing the document gives every document |

- A definition and a listed import of the same name, or two listed imports of one name, are `F013`.
- A name from a `"*"` import that you also define or list is yours, without a diagnostic. So a kit or pack that adds a component later never changes what a name you chose means.
- The same name from two `"*"` imports, or from one and the built-in components, is `F023` when you use it. List the one you mean to settle it.
- A definition or listed import named like a built-in component, such as `grid`, replaces it, with `W112`. `if`, `each`, `value` and `box` can't be replaced (`F013`).

## The signature

A signature is written in YAML, as the frontmatter is. It has up to three keys:

| Key | Holds |
| --- | --- |
| `params` | each param's name and its declaration: its type, and its default if it has one |
| `slots` | the slots, a list of names (chapter 13), or names with options (chapter 14) |
| `headings` | `relative`, the default, or `absolute` |

`headings: absolute` keeps the body's headings at the levels written, for a component such as a title page, whose `#` must stay a first-level heading wherever it is used. Without it, headings follow where the component is used (chapter 12).

## Types

A declaration starts with the type of the value:

| Type | Takes | Written in a use |
| --- | --- | --- |
| `string` | text | `title="Work day"`, or `title=Saturday` for one word |
| `number` | a number | `number=4`, `ratio=0.5` |
| `boolean` | true or false | `open=true`, or `open` alone for true |
| `date` | a date, year, month and day | `day=2026-11-07` |
| `array` | a list | `plots=(md.doc.plots)` |
| `object` | a record | `editor=(md.doc.editor)` |
| an enum, `small \| large` | one of the names listed | `size=large` |
| `name` | a name, for a template slot (chapter 14) | `as=plot` |

A bare value is read as its param's type: `number=4` is a number, `day=2026-11-07` a date, `size=large` checked against the enum's names. A value in quotes is always text, so `number="4"` is `F009`, as is `size=medium` or `open=yes`. An expression's value must have the param's type too, or it is `F024`.

The values of the layout params have types of their own, for a component that passes a param on to its box: `space` (`s`, `m`, `l` or a length), `length`, `color`, `surface` (a background), `border`, `radius`, `tracks` and `areas`. A component fenced `{display=flex gap=(gap)}` with `gap: space = s` lets each use choose its gap.

A type misspelt, such as `numbr`, is `F037`, which suggests the right one. The param then takes any value, so one mistake gives one diagnostic, not one for every use.

## Required, optional and defaults

| Declared | A use without it |
| --- | --- |
| `number: number` | is `F010`: the param is required, and the component shows its fallback |
| `gardener: string?` | gives no value: in the body, `gardener == null` is true, and `{{gardener}}` writes nothing |
| `crop: string = "something new"` | gives the default, *something new* |
| `size: small \| large = small` | gives `small` |

A default is written as a value in a use is: `= 3`, `= small`, `= "two words"` in quotes when it has a space.

Plot 3's card shows how they work together: its use gives only `number`, so `gardener` has no value, the body's `if` shows *Free for next year*, and `size` is `small`.

## The long form

A declaration can also be a record, for what a line can't say well:

```text
tone:
  type: plain | urgent
  default: plain
  doc: An urgent notice has the accent background.
```

`type` is the type, with a `?` if it's optional, `default` the default, and `doc` a sentence about the param, for the people who use the component and the tools they write with.

## Inputs

The reader's inputs (chapter 7) are declared in the same way, under `md.inputs`: a name, a type and a default. A component can't read them itself; the document passes an input's value to it as a param, and the component shows it again whenever the reader changes it:

````md
---
md:
  inputs:
    size: all | small | large = all
---

```compmd component=showing
params:
  size: all | small | large
---
Showing {{size}} plots.
```

:: select-input[Show: all plots] {bind=size}

:: showing[Showing all plots] {size=(md.input.size)}
````

Signatures have one more type, `input<…>`, for a component that changes an input itself, such as a chart whose bars the reader clicks. Only components written in code use it, so it is [the Developer's Guide](../developers/README.md)'s.

## Fallbacks

A component written in CompMD can be drawn by every app that shows CompMD, so it shows its fallback only when something is wrong, or in a plain Markdown viewer. What shows in its place:

| Component | Its fallback |
| --- | --- |
| a leaf, or an inline component | the text in its brackets |
| a container | its slots, the default one first and then the named ones in the order written, as plain Markdown |

So nothing you wrote is lost: a broken `work-day` still shows its jobs and what to bring, without its heading and its box.

A component shows its fallback when:

- a required param is missing (`F010`), or a required param's value is wrong (`F009`), so the param is left out;
- its name is unknown (`F002`), or it leads back to itself (`F019`);
- the file or pack it comes from can't be had (`F026`);
- it is used inline, and its body isn't one paragraph.

A param that isn't required and has a wrong value is left out instead, with its diagnostic, and the component is drawn with the param's default.

In a plain Markdown viewer, such as GitHub's, every leaf and inline component shows its fallback, every container its content, and every definition its fence, as code. Writing each fallback as what the component shows, *Plot 3, small: free for next year*, keeps the newsletter readable there too.

Next: Part V, [using a pack](16-using-a-pack.md), for components other people publish.
