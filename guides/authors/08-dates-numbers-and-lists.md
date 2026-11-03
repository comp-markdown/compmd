# 8. Dates, numbers and lists

Chapters 4 and 5 used a few functions to write a date or filter a list. This chapter has the rest: counting days, sums and averages, sorting, grouping, and writing it all for readers. The newsletter gets the next work day, counted from today, and the season's harvest, added up from a spreadsheet.

## The newsletter, by the numbers

```md snippet=lists/garden.comp.md
---
md:
  version: 1
  data:
    - from: ./harvest.csv
      as: harvest
      schema: {date: date, plot: number, crop: string, kg: number}
title: Elm Street Garden
issue: November 2026
workdays: ["2026-11-07", "2026-11-21", "2026-12-05"]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}, for the week of :value[October 25, 2026]{of=(md.date(md.startOf(md.today(), "week"), "long"))}.

## The next work day

::: each {in=(md.slice(md.doc.workdays.filter(d, d >= md.today()), 0, 1)) as=next}

:value[Saturday, November 7, 2026]{of=(md.date(next, "full"))}, in :value[7 days]{of=(string(md.daysBetween(md.today(), next)) + " " + md.plural(md.daysBetween(md.today(), next), "day", "days"))}. After it: :value[November 21, 2026 and December 5, 2026]{of=(md.list(md.doc.workdays.filter(d, d > next).map(d, md.date(d, "long"))))}.

:: #empty

No more work days this year. See you in the spring.

::: /each

## The harvest

Since :value[September 12]{of=(md.date(md.sort(md.data.harvest.map(h, h.date))[0], "long"))}, the plots gave :value[30.0 kg]{of=(md.number(md.sum(md.data.harvest.map(h, h.kg)), 1) + " kg")} in :value[7 pickings]{of=(string(size(md.data.harvest)) + " " + md.plural(size(md.data.harvest), "picking", "pickings"))}, :value[4.3 kg]{of=(md.number(md.sum(md.data.harvest.map(h, h.kg)) / size(md.data.harvest), 1) + " kg")} each on average. The biggest was :value[9.0 kg]{of=(md.number(md.max(md.data.harvest.map(h, h.kg)), 1) + " kg")} of :value[squash]{of=(md.reverse(md.sortBy(md.data.harvest, "kg"))[0].crop)}.

::: each {in=(md.sort(md.unique(md.data.harvest.map(h, h.crop)))) as=crop join=list}

- **{{crop}}**: :value[7.7 kg]{of=(md.number(md.sum(md.groupBy(md.data.harvest, "crop")[crop].map(h, h.kg)), 1) + " kg")} from :value[2 pickings]{of=(string(size(md.groupBy(md.data.harvest, "crop")[crop])) + " " + md.plural(size(md.groupBy(md.data.harvest, "crop")[crop]), "picking", "pickings"))}

::: /each

The last three:

::: each {in=(md.slice(md.reverse(md.sortBy(md.data.harvest, "date")), 0, 3)) as=h join=list}

1. :value[October 24]{of=(md.date(h.date, "long"))}: {{h.crop}} from plot {{h.plot}}, {{h.kg}} kg

::: /each

Bring your spare :value[beans, kale, leeks, or squash]{of=(md.list(md.sort(md.unique(md.data.harvest.map(h, h.crop))), "or"))} to the swap table by the gate.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/lists) in the playground.

The harvest, one row per picking:

```csv snippet=lists/harvest.csv
date,plot,crop,kg
2026-09-12,1,beans,3.5
2026-09-19,5,beans,4.2
2026-09-26,2,kale,2
2026-10-03,4,leeks,5.5
2026-10-10,2,kale,1.8
2026-10-17,1,squash,9
2026-10-24,4,leeks,4
```

Made on October 31, the page says the next work day is *Saturday, November 7, 2026, in 7 days*, then lists the harvest: *30.0 kg in 7 pickings, 4.3 kg each on average*, a line per crop (*beans: 7.7 kg from 2 pickings*), and the last three pickings, newest first. Add a row to the spreadsheet, and every figure follows. After December 5, the work days' `:: #empty` slot shows instead.

## Dates

A date is text written as year, month and day, `"2026-11-07"`, in the frontmatter, in data and in inputs alike. Dates written this way sort and compare as they should, so `d >= md.today()` is true for today and the days after it, as `filter` uses it above.

| Function | Gives | Example, on October 31, 2026 |
| --- | --- | --- |
| `md.today()` | today's date | *2026-10-31* |
| `md.addDays(date, n)` | the date `n` days later, earlier for a negative `n` | `md.addDays(md.today(), -7)` is *2026-10-24* |
| `md.daysBetween(a, b)` | the days from `a` to `b`, negative when `b` is before `a` | `md.daysBetween(md.today(), "2026-11-07")` is *7* |
| `md.startOf(date, unit)` | the first day of the date's `"week"`, `"month"` or `"year"` | `md.startOf(md.today(), "month")` is *2026-10-01* |

A week starts on the day the document's language starts it: Sunday in English, so `md.startOf(md.today(), "week")` is *2026-10-25*, and Monday in German.

*Today* is the day the page is made, wherever that is. A page made once and kept, such as a printed one, keeps the day it was made. Within one page, every `md.today()` is the same day.

### Writing a date

`md.date(date, style)` writes a date for readers, in one of four styles:

| Style | Gives |
| --- | --- |
| `"short"` | *11/7/26* |
| `"medium"` | *Nov 7, 2026* |
| `"long"` | *November 7, 2026* |
| `"full"` | *Saturday, November 7, 2026* |

The document's language decides the order and the words (chapter 4): with `lang: de`, `"long"` gives *7. November 2026*. `{{…}}` writes a date as it is stored, *2026-11-07*.

### Times of day

Most documents need only dates. For the few that need the hour, `md.now()` is the moment the page is made, a *timestamp*, and its getters read it in the document's time zone, which `tz` under `md` sets:

```md
---
md:
  tz: Europe/London
---

:value[Good morning]{of=(md.now().getHours() < 12 ? "Good morning" : "Good afternoon")}, from the garden.
This page was made on :value[October 31, 2026]{of=(md.date(md.now(), "long"))}.
```

| Written | Gives |
| --- | --- |
| `md.now().getHours()` | the hour, 0 to 23 |
| `md.now().getMinutes()` | the minute |
| `md.now().getDayOfWeek()` | the day of the week, 0 for Sunday |
| `md.dateOf(md.now())` | the date of a timestamp: *2026-10-31* |

Without `tz`, the time zone is the app's, usually the reader's. A `tz` is a name such as `Europe/London` or `America/Chicago`, and it sets `md.today()` too.

## Numbers

Numbers in the frontmatter, in data and in inputs can have decimals: `4.2`, `12`. Some numbers are always whole: counts, such as `size(…)` and `md.daysBetween(…)`, `md.loop`'s, and numbers you write without a point. The two mix in sums and comparisons, `md.input.plots * 25` and `size(rows) > 2.5`, with two things to know:

- **Dividing two whole numbers drops the rest.** `size(md.data.harvest) / 2` with 7 pickings is 3. Make one of them a number with decimals first: `double(size(md.data.harvest)) / 2` is 3.5. A number with decimals divided by a whole one is fine, as the average above is.
- **A position in a list is whole.** `md.doc.helpers[0]` is the first helper; with a position from an input, write `md.doc.helpers[int(md.input.n)]`.

`int(n)` drops the decimals, `double(n)` gives a number with them, and `string(n)` writes it as text, as `{{…}}` would: *4.2*, *12*.

### Writing numbers

| Function | Example | Gives |
| --- | --- | --- |
| `md.number(n, digits)` | `md.number(1234.567, 1)` | *1,234.6*: grouped, rounded to that many decimals |
| `md.currency(n, code)` | `md.currency(1234.5, "EUR")` | *€1,234.50*, with the currency's own decimals |

`md.number` rounds a half up, away from zero: `md.number(2.5, 0)` is *3*. The currency is a three-letter code, `"USD"`, `"EUR"`, `"GBP"`. Both follow the document's language: with `lang: de`, `md.number(1234.567, 1)` is *1.234,6*.

### Sums, smallest and largest

These take a list of numbers, so a list of records goes through `map` first, as the harvest does: `md.data.harvest.map(h, h.kg)`.

| Function | Gives |
| --- | --- |
| `md.sum(list)` | the sum; `0` for an empty list |
| `md.min(list)`, `md.max(list)` | the smallest, the largest; `null` for an empty list |

An average is the sum over the count: `md.sum(…) / size(…)`. Check that a list isn't empty before dividing by its size, or writing its `md.max`: `null` can't be written with `string()`, and is a diagnostic (`F024`).

## Plurals

`md.plural(n, one, other)` picks a word by a number: `md.plural(1, "picking", "pickings")` is *picking*, and any other number gives *pickings*. Write the number beside it yourself, as the newsletter does.

Some languages have more forms than two. Give them by their names, `one`, `few`, `many` and `other`, and the document's language picks; `other` must be there:

```md
---
md:
  lang: ru
plots: 5
---

На участке :value[5 грядок]{of=(string(md.doc.plots) + " " + md.plural(md.doc.plots, {"one": "грядка", "few": "грядки", "many": "грядок", "other": "грядки"}))}.
```

With 1 it writes *грядка*, with 3 *грядки*, with 5 *грядок*. The names are the [CLDR plural categories](https://cldr.unicode.org/index/cldr-spec/plural-rules), which also has `zero` and `two` for the languages that use them.

## Lists

Chapter 5 chose items with `filter`, `exists`, `all` and `map`, written after a list. Two more of that kind:

| Written | Gives |
| --- | --- |
| `list.exists_one(p, p.crop == "garlic")` | true when the condition is true for exactly one item |
| `"garlic" in md.doc.crops` | true when the list holds the value |

And `size(list)` counts. The rest are functions that take a list:

| Function | Gives |
| --- | --- |
| `md.sort(list)` | the values in order, smallest or earliest first |
| `md.sortBy(list, "field")` | the records in order of one field: `md.sortBy(md.data.harvest, "kg")` |
| `md.reverse(list)` | the list turned around: largest or newest first, after a sort |
| `md.slice(list, start, end)` | the items from position `start` up to, not including, `end`: `md.slice(list, 0, 3)` is the first three, fewer if the list is shorter |
| `md.unique(list)` | each value once, in the order they first come |
| `md.groupBy(list, "field")` | the records grouped by one field's value, below |
| `md.list(list)` | the values written as a sentence: *beans, kale, and leeks* |
| `md.list(list, "or")` | the same with *or*: *beans, kale, or leeks* |

A few things the newsletter relies on:

- **The first item.** `list[0]` is the first item, and an error on an empty list. `md.slice(list, 0, 1)` is a list of at most one item, so an `each` over it shows the item, or its `:: #empty` slot when there is none, as the next work day does.
- **Sorting.** Sorting keeps items that are equal in the order they came. Text sorts by its letters' codes, not by a dictionary: capitals before small letters, and letters with accents after *z*. Write the values you sort the same way, all small letters or all capitalized.
- **Sorting by a computed value.** `md.sortBy` takes a field's name, not a formula. To sort by something computed, `map` the records into ones with that field first: `md.sortBy(md.data.harvest.map(h, {"crop": h.crop, "grams": h.kg * 1000}), "grams")`.

### Groups

`md.groupBy(md.data.harvest, "crop")` makes a record with one entry per crop, each holding that crop's rows, in the order the crops first come. Read one group by its name in brackets: `md.groupBy(md.data.harvest, "crop")["kale"]` is kale's two pickings.

`each` goes through a list, not a record, so the newsletter goes through the crops' names, `md.sort(md.unique(md.data.harvest.map(h, h.crop)))`, and reads each crop's group with `[crop]`: its sum, and its size. The same could be written with `filter`, `md.data.harvest.filter(h, h.crop == crop)`; the group says it once.

## Text

Text has functions of its own, written after it:

| Written | Gives |
| --- | --- |
| `"Plot 4".contains("4")` | true when the text holds the other |
| `"Plot 4".startsWith("Plot")`, `.endsWith("4")` | true when it starts or ends so |
| `"garlic".upperAscii()`, `"KALE".lowerAscii()` | *GARLIC*, *kale*: only the letters A to Z change |
| `" kale ".trim()` | *kale*, without the spaces around it |
| `"beans, kale".split(", ")` | a list, *beans* and *kale* |
| `md.doc.rotation.join(" → ")` | the list's items joined into one text |
| `"garlic".substring(0, 3)` | *gar*, from position 0 up to 3 |
| `"garlic".replace("g", "G")` | *GarliG*: every match |
| `"garlic".indexOf("l")` | *3*, where it first is; -1 when it isn't |

There is no matching by patterns: a function for that would work differently in different apps.

## When a function is given the wrong thing

A function given something it can't use is a diagnostic, `F024`, that says what it takes: `md.date(d, "longer")` says the styles are `"short"`, `"medium"`, `"long"` and `"full"`, and `md.sum` over text says it needs numbers. Text from a CSV without a schema is the usual cause: chapter 6's schema makes its numbers and dates what they are.

Every function is in the [spec's expression profile](https://compmd.hakanserce.com/spec/#82-expressions).

Next: [boxes, rows, stacks and grids](09-boxes-rows-stacks-and-grids.md), the first chapter of Part III, where the newsletter's blocks go side by side.
