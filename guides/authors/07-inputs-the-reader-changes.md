# 7. Inputs the reader changes

So far the page shows what the document and its files say. An *input* is a value the reader changes on the page: a name typed in, a size picked, a box ticked. Everything computed from it follows at once. This chapter gives the newsletter a sign-up form that works out the year's fee, and a list of jobs that loads the month the reader picks.

## The newsletter, with a sign-up form

```md snippet=inputs/garden.comp.md
---
md:
  version: 1
  inputs:
    name: string = ""
    size: small | large = small
    plots: number = 1
    tools: boolean = false
    start: date = (md.doc.workday)
    month: november | december = november
  data:
    - from: ./jobs-{{md.input.month}}.yaml
      as: jobs
title: Elm Street Garden
issue: November 2026
workday: "2026-11-07"
fees: {small: 25, large: 40, tools: 5}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Take a plot for next year

::: grid {min=14rem gap=m}
:: text-input[Your name: none yet] {bind=name}
:: select-input[Size: small] {bind=size}
:: number-input[Plots: 1] {bind=plots}
:: toggle-input[Share the tool shed: no] {bind=tools}
:: date-input[First work day: November 7, 2026] {bind=start}
::: /grid

::: box {padding=m border=subtle border-radius=m}

::: if {when=(md.input.name != "")}

Thank you, **{{md.input.name}}**.

::: /if

:value[1 small plot]{of=(md.number(md.input.plots, 0) + " " + md.input.size + " " + md.plural(md.input.plots, "plot", "plots"))} for the year: :value[$25.00]{of=(md.currency(md.doc.fees[md.input.size] * md.input.plots + (md.input.tools ? md.doc.fees.tools : 0.0), "USD"))}, :value[without the tool shed]{of=(md.input.tools ? "with the tool shed" : "without the tool shed")}. Your first work day is :value[Saturday, November 7, 2026]{of=(md.date(md.input.start, "full"))}.

::: /box

## Jobs by month

:: select-input[Month: november] {bind=month}

::: each {in=(md.data.jobs) as=job join=list}

- {{job}}

::: /each
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/inputs) in the playground. Type a name, pick *large*, make it 2 plots and tick the tool shed: the box says *Thank you*, then *2 large plots for the year: $85.00, with the tool shed*. Pick *december*, and the jobs change.

The jobs for each month are a file beside the document:

```yaml snippet=inputs/jobs-november.yaml
- Turn the compost
- Plant the garlic
- Mend the gate by plot 4
```

```yaml snippet=inputs/jobs-december.yaml
- Cover the empty beds
- Oil the tools
- Order next year's seeds
```

## Declaring an input

Inputs are declared in the frontmatter, under `md.inputs`, one per line: a name, a type, `=` and the value it starts with, its *default*.

```yaml
md:
  inputs:
    plots: number = 1
```

The name is one word, or words joined by `_`, as an `each` name is (chapter 5). The page reads the input as `md.input.plots`, in `{{…}}` and in expressions, as it reads `md.doc`.

Give every input a default: it is what the page shows before the reader changes anything, and what a reader without an interactive page sees.

## The types, and their components

Each type has a built-in component that shows the input and lets the reader change it:

| Type | Written | Its component | The reader |
| --- | --- | --- | --- |
| text | `name: string = ""` | `text-input` | types text |
| a number | `plots: number = 1` | `number-input` | types a number |
| true or false | `tools: boolean = false` | `toggle-input` | ticks a box |
| a date | `start: date = "2026-11-07"` | `date-input` | picks a day |
| one of a few words | `size: small \| large = small` | `select-input` | picks one from a list |

The last is a *choice* (an enum): its words, separated by `|`, are what the list offers. Each is a name, so it is written without quotes, and starts with a letter. A choice with spaces in it needs the whole line quoted: `status: '"to do" | done = done'`.

A default can be an expression in parentheses, computed when the page is made. It sees the frontmatter and the `md.` functions, but not data or other inputs:

| Written | Starts as |
| --- | --- |
| `start: date = (md.doc.workday)` | the frontmatter's work day |
| `start: date = (md.today())` | today |
| `start: date = (md.addDays(md.today(), 14))` | two weeks from today |

## Showing an input

An input component is a leaf. `bind` names the input it changes, and its brackets hold its fallback:

```text
:: number-input[Plots: 1] {bind=plots}
```

Write the fallback as a label, a colon, and the default: *Plots: 1*. A plain renderer, such as GitHub's, shows it as it is, so the text still tells the reader what the value is. The playground uses the part before the colon as the control's label, *Plots*, next to a box holding 1.

`bind` must name an input the document declares, of a type the component edits: a `text-input` bound to `plots` is a diagnostic, `F030`, that says *\`bind\` binds a string, but \`plots\` is a number*. So is a `bind` to an input no one declared.

The common params of chapter 3 work as on any block: `:: number-input[Plots: 1] {bind=plots max-width=12rem}`. The grid in the newsletter puts the controls side by side, as many as fit (Part III).

The same input can have two components, or none. The playground shows every input of a document in a bar above its tabs, so you can try other values there too.

## Using an input

An input is a value like any other, so everything in chapters 4 and 5 works with it:

- `{{md.input.name}}` writes it.
- `:value[…]{of=(…)}` computes with it: `md.doc.fees[md.input.size]` looks up the fee under the name the reader picked, and the `? :` adds the tool shed's fee when the box is ticked.
- `::: if {when=(md.input.name != "")}` shows the thanks only once a name is typed.
- A param can follow it: `background=(md.input.tools ? "accent" : "subtle")`.

When the reader changes an input, the page is made again with the new value: every part that uses it changes, and nothing else does. There is nothing else to write, no events and no scripts: values flow one way, from the inputs to what the page shows.

A number input holds any number the reader types, 1.5 too. `md.number(md.input.plots, 0)` writes it as a whole number, and the fee is computed from what was typed.

## Loading by an input

A data import's `from` can use inputs, so the reader's choice picks the file:

```yaml
md:
  inputs:
    month: november | december = november
  data:
    - from: ./jobs-{{md.input.month}}.yaml
      as: jobs
```

With *december* picked, the document loads `./jobs-december.yaml`. A `from` can use `md.input` and `md.doc`, nothing else: never `md.data`, so data from one place is never sent to another (`F031` otherwise). Each value fills one part of the path: it can't add a `/` or a `..`, so the file stays in its folder.

The same works for an address on a website (chapter 6): `from: https://weather.example.org/frost?street={{md.input.street}}`. The site's address itself must be written out, so the app can check it before anything is fetched; only the part after it may hold inputs. The site then learns what the reader picked, since it is part of the address.

## Where the values come from

On the page, from the reader. An app can also give the values itself, for example from the page's address, and keep them between visits; how is the app's business. A value that doesn't fit its input's type is ignored, and the default is used, so a page never shows a number input holding a word.

Where nothing is interactive, a plain renderer or a printout, the page shows the defaults, and the input components show their fallbacks or their values.

## Inputs and components

Inputs belong to the document being read. A component of your own (Part IV) doesn't see `md.input`: give it the value as a param, `:: fee-table[…] {size=(md.input.size)}`. An input of another document that this one loads (chapter 6) isn't used.

A component from a pack can change an input too, when its params say so and you bind one to it: a chart whose bars the reader clicks to pick a region, written `selected=region`. Its README says which params do. Since such a component's code can set the input, that input can't be used in a data import's `from` (`F031`): code could otherwise send what it likes through it.

The [spec's inputs](https://compmd.hakanserce.com/spec/#87-inputs) have every rule.

Next: [dates, numbers and lists](08-dates-numbers-and-lists.md), for the functions that count, sum, sort and write dates.
