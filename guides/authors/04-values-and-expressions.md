# 4. Values and expressions

A document can say a thing once and use it everywhere: the issue's date in the heading and the notice, a fee in the table and in the text. This chapter shows how to insert values from the frontmatter, how to compute and format them, and how a param can follow a value. The newsletter's dates, fees and names now come from its frontmatter.

## The newsletter, with values

```md snippet=values/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
editor: {name: Maya Okafor, phone: 555-0142}
workday: "2026-11-07"
water_off: "2026-11-15"
fees: {small: 25, large: 40}
plots: {small: 12, large: 8, taken: 17}
helpers: [Maya, Tomás, Priya, Jun]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}, from {{md.doc.editor}}.

::: box {padding=m border-radius=m background=(md.daysBetween(md.today(), md.doc.water_off) <= 14 ? "accent" : "subtle")}

**The water goes off on :value[November 15, 2026]{of=(md.date(md.doc.water_off, "long"))}** for the winter. Empty your cans and hoses before then.

::: /box

## Work day

:value[Saturday, November 7, 2026]{of=(md.date(md.doc.workday, "full"))}, from nine to noon, with :value[Maya, Tomás, Priya, and Jun]{of=(md.list(md.doc.helpers))} leading the jobs.

## Plots for next year

| Size | Plots | Fee for the year |
| --- | ---: | ---: |
| Small | {{md.doc.plots.small}} | :value[$25.00]{of=(md.currency(md.doc.fees.small, "USD"))} |
| Large | {{md.doc.plots.large}} | :value[$40.00]{of=(md.currency(md.doc.fees.large, "USD"))} |

:value[3 plots are]{of=(string(md.doc.plots.small + md.doc.plots.large - md.doc.plots.taken) + " " + md.plural(md.doc.plots.small + md.doc.plots.large - md.doc.plots.taken, "plot is", "plots are"))} still free. Call {{md.doc.editor.name}} at {{md.doc.editor.phone}} to take one.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/values) in the playground.

## The frontmatter's values

The frontmatter is YAML, and a value can be more than a line of text:

| Written | Is |
| --- | --- |
| `title: Elm Street Garden` | text |
| `fees: {small: 25, large: 40}` | a record: values under names, written in braces or one per line, indented |
| `helpers: [Maya, Tomás, Priya, Jun]` | a list, in brackets or one `- item` per line |
| `small: 25` | a number |
| `workday: "2026-11-07"` | a date, written as year, month and day |
| `open: true` | `true` or `false` |

Quote a value that would otherwise read as something else: a time such as `"9:30"`, or a version such as `"1.10"`, which unquoted is the number 1.1.

## Inserting a value

`{{…}}` inserts a value where it stands. Inside it goes a *path*: where the value is, written as names joined by dots.

- `md.doc` is your frontmatter, so `{{md.doc.issue}}` is the `issue` key: *November 2026*.
- A dot goes into a record: `{{md.doc.plots.small}}` is *12*, and `{{md.doc.editor.phone}}` is *555-0142*.
- A record with a `name` reads as its name, since it stands for someone or something: `{{md.doc.editor}}` is *Maya Okafor*.
- A number is written as it is (*25*, *12.5*), and a date as you wrote it (*2026-11-07*). To write them for readers, *$25.00* or *November 7*, use `value`, below.

A list isn't inserted as it is, since there are many ways to write one; `{{md.doc.helpers}}` is a diagnostic (`F024`). Write it with `md.list`, below, or one item at a time with `each` (chapter 5).

`{{…}}` works in text, in headings, in table cells, in a link's address (`[the rules](https://example.org/{{md.doc.rules}})`), and in params written in quotes (`title="Plot {{md.doc.plot}}"`). It doesn't work in code, which always shows what you wrote.

A path that doesn't exist, such as `{{md.doc.isue}}`, is a diagnostic (`F014`) that suggests the nearest one, here `md.doc.issue`. To write double braces as text, start them with a backslash: `\{{like this}}`.

## Computed text: `value`

A path only finds a value; it can't do anything to it. For text that is computed or formatted, use the inline component `value`. Its `of` param is the text to show, and its brackets hold the fallback, the text a plain renderer shows instead:

```md
---
fees: {small: 25, large: 40}
---

A small plot is :value[$25.00]{of=(md.currency(md.doc.fees.small, "USD"))} a year.
```

The fallback should be what the value usually is, so the text still reads well on GitHub. It doesn't change when the frontmatter does, so it's a sample, not a promise.

## Expressions

The `(…)` in `of=(…)` is an *expression*: a small formula, computed when the page is made. Any param can be one, not only `of`. In the notice above, `background` is computed: the accent color when the water goes off within two weeks, the subtle one before.

Expressions are written in [CEL](https://cel.dev), a small language for formulas. You'll need only a little of it:

| Written | Is |
| --- | --- |
| `md.doc.fees.small * 4` | arithmetic: `+`, `-`, `*`, `/` |
| `md.doc.plots.taken < 20` | a comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` |
| `a && b`, `a \|\| b`, `!a` | and, or, not |
| `soon ? "accent" : "subtle"` | one value or another: if `soon` is true, `"accent"`, otherwise `"subtle"` |
| `"Plot " + md.doc.plot` | text joined to text |
| `string(md.doc.fees.small)` | a number as text, *25* |
| `size(md.doc.helpers)` | how many items a list has |

Text in an expression is in double quotes. Paths work as in `{{…}}`: `md.doc.fees.small` is the small plots' fee.

`value` shows text, so a number goes through `string(…)` or one of the formatting functions below first: `of=(string(size(md.doc.helpers)))`, not `of=(size(md.doc.helpers))`.

A mistake in an expression is a diagnostic (`F024`) that points to the place in the expression, and for a misspelt name, suggests the right one.

## Formatting for readers

CompMD's own functions start with `md.`. These format values for readers:

| Function | Example | Gives |
| --- | --- | --- |
| `md.date(date, style)` | `md.date(md.doc.workday, "long")` | *November 7, 2026*; the styles are `"short"`, `"medium"`, `"long"` and `"full"` |
| `md.currency(n, code)` | `md.currency(md.doc.fees.large, "USD")` | *$40.00* |
| `md.number(n, digits)` | `md.number(12.345, 1)` | *12.3* |
| `md.plural(n, one, other)` | `md.plural(3, "plot", "plots")` | *plots* |
| `md.list(list)` | `md.list(md.doc.helpers)` | *Maya, Tomás, Priya, and Jun* |

And these work with dates:

| Function | Gives |
| --- | --- |
| `md.today()` | today's date, when the page is made |
| `md.daysBetween(a, b)` | the days from `a` to `b`: `md.daysBetween(md.today(), md.doc.water_off)` |
| `md.addDays(date, n)` | the date `n` days later, or earlier for a negative `n` |

Chapter 8 has the rest, for dates, numbers and lists.

**The document's language** decides how they write: with `lang: de` under `md` in the frontmatter, `md.date(md.doc.workday, "long")` gives *7. November 2026*, and `md.currency(25, "EUR")` gives *25,00 €*.

Next: [showing and repeating](05-showing-and-repeating.md), for parts that show only sometimes, and lists made from the frontmatter.
