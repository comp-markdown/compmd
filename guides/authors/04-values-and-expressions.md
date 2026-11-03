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
- `md.lang` is the document's language, `en` unless the frontmatter says otherwise (below).
- A number is written as it is (*25*, *12.5*), and a date as you wrote it (*2026-11-07*). To write them for readers, *$25.00* or *November 7*, use `value`, below.

`true` and `false` are written as they are, and an empty value (`note:` with nothing after it) as nothing. A list isn't inserted as it is, since there are many ways to write one; `{{md.doc.helpers}}` is a diagnostic (`F024`), and so is a record without a `name`. Write it with `md.list`, below, or one item at a time with `each` (chapter 5).

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
| `md.doc.fees.small * 4` | arithmetic: `+`, `-`, `*`, `/`, and `%` for the remainder |
| `md.doc.plots.taken < 20` | a comparison: `==`, `!=`, `<`, `<=`, `>`, `>=` |
| `a && b`, `a \|\| b`, `!a` | and, or, not |
| `soon ? "accent" : "subtle"` | one value or another: if `soon` is true, `"accent"`, otherwise `"subtle"` |
| `"Plot " + md.doc.plot` | text joined to text |
| `string(md.doc.fees.small)` | a number as text, *25* |
| `size(md.doc.helpers)` | how many items a list has |
| `md.doc.helpers[0]` | an item of a list by its place, counting from 0: *Maya* |
| `"Jun" in md.doc.helpers` | true when the list holds the value |
| `["Maya", "Jun"]` | a list, written in the expression |
| `has(md.doc.subtitle)` | true when the frontmatter has the key |

Text in an expression is in double quotes, or single ones: `'subtle'`. Paths work as in `{{…}}`: `md.doc.fees.small` is the small plots' fee. Parentheses group, as in arithmetic: `(md.doc.plots.small + md.doc.plots.large) * 2`. An expression is written on one line.

`value` shows text, so a number goes through `string(…)` or one of the formatting functions below first: `of=(string(size(md.doc.helpers)))`, not `of=(size(md.doc.helpers))`.

Text has a few functions of its own, written after it with a dot:

| Written | Gives |
| --- | --- |
| `md.doc.title.contains("Garden")` | true when the text holds the other; also `startsWith` and `endsWith` |
| `md.doc.title.upperAscii()` | *ELM STREET GARDEN*; `lowerAscii()` for small letters (A to Z only) |
| `md.doc.title.replace("Street", "St.")` | *Elm St. Garden* |
| `" Elm ".trim()` | the text without spaces at its ends: *Elm* |
| `"a,b".split(",")` | a list of the parts: `["a", "b"]` |
| `md.doc.helpers.join(", ")` | the items of a list of text, joined: *Maya, Tomás, Priya, Jun* |

### Keys that may be missing

A path to a key the frontmatter doesn't have is a mistake, in `{{…}}` and in expressions alike. When a key is there in some issues and not in others, ask first with `has`, and read it only when it's there:

```md
---
title: Elm Street Garden
---

# :value[Elm Street Garden]{of=(has(md.doc.subtitle) ? md.doc.title + ": " + md.doc.subtitle : md.doc.title)}
```

### Numbers, text and true or false

An expression knows what kind of value each part is, and checks that they fit before the page is made:

- **Text and numbers don't mix.** `"Plot " + 4` is a mistake: write `"Plot " + string(4)`. Comparing them is one too.
- **Whole numbers.** A number written in an expression without a decimal point, `4`, and what `size(…)` and `md.loop` give, are whole numbers, and dividing two of them drops the remainder: `size(md.doc.helpers) / 3` is *1* with four helpers. Write `double(size(md.doc.helpers)) / 3` for *1.333…*. A number from the frontmatter is never whole in this sense, so `md.doc.fees.small / 2` is *12.5*. A list's place must be a whole number, and `int(…)` makes one: `md.doc.helpers[int(md.doc.plots.taken) % 4]`.
- **Dates are text**, written year, month and day, so they compare as dates do: `md.doc.workday < md.doc.water_off` is true.
- **`true` and `false`** are what comparisons give, and what `when` takes in chapter 5. `if` and `?` take nothing else: a number or a text is a mistake there, never a stand-in for true.

### When an expression is wrong

A mistake in an expression is a diagnostic (`F024`) that points to the place in it, and says what is wrong:

| Written | The diagnostic says |
| --- | --- |
| `md.doc.fess.small` | unknown field 'fess'; did you mean 'fees'? |
| `"Fee " + md.doc.fees.small` | '+' can't be applied to string and double |
| `of=(size(md.doc.helpers))` | `of` takes string, but the expression gives int |

*string*, *double* and *int* are the expression's words for text, a number and a whole number. Where the mistake is, the component shows its fallback, or the param is left out as if you hadn't written it, so the rest of the page is still there.

`{{…}}` takes only a path, never a formula: `{{ md.doc.fees.small * 4 }}` shows as written, with a warning (`W106`) that suggests `value`.

## Formatting for readers

CompMD's own functions start with `md.`. These format values for readers:

| Function | Example | Gives |
| --- | --- | --- |
| `md.date(date, style)` | `md.date(md.doc.workday, "long")` | *November 7, 2026*; the other styles give *11/7/26* (`"short"`), *Nov 7, 2026* (`"medium"`) and *Saturday, November 7, 2026* (`"full"`) |
| `md.currency(n, code)` | `md.currency(md.doc.fees.large, "USD")` | *$40.00*; the code is the currency's three letters, such as `"EUR"` or `"GBP"` |
| `md.number(n, digits)` | `md.number(1636.5, 2)` | *1,636.50*: rounded to that many digits after the point, with the language's separators |
| `md.plural(n, one, other)` | `md.plural(3, "plot", "plots")` | *plots*: the first word for one, the second for any other number |
| `md.list(list)` | `md.list(md.doc.helpers)` | *Maya, Tomás, Priya, and Jun*; `md.list(md.doc.helpers, "or")` gives *Maya, Tomás, Priya, or Jun* |

And these work with dates:

| Function | Gives |
| --- | --- |
| `md.today()` | today's date, when the page is made, in the document's time zone, `tz` under `md` ([chapter 8](08-dates-numbers-and-lists.md)), or else the app's |
| `md.daysBetween(a, b)` | the days from `a` to `b`: `md.daysBetween(md.today(), md.doc.water_off)` |
| `md.addDays(date, n)` | the date `n` days later, or earlier for a negative `n` |

Chapter 8 has the rest, for dates, numbers and lists.

**The document's language** decides how they write: with `lang: de` under `md` in the frontmatter, `md.date(md.doc.workday, "long")` gives *7. November 2026*, and `md.currency(25, "EUR")` gives *25,00 €*.

Next: [showing and repeating](05-showing-and-repeating.md), for parts that show only sometimes, and lists made from the frontmatter.
