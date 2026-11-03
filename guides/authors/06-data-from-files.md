# 6. Data from files

The frontmatter holds a few values well, but a list of plots belongs in a spreadsheet, and the club's fees in one place every issue reads. This chapter loads data from files beside the document: a CSV file of the plots, a YAML file of the work days, and another CompMD document's frontmatter. It also says what happens when a file is missing or a row is wrong, and where else data can come from.

## The newsletter, from its files

The document:

```md snippet=data/garden.comp.md
---
md:
  version: 1
  data:
    - from: ./plots.csv
      as: plots
      schema:
        number: {from: Plot, type: number}
        gardener: {from: Gardener, type: string?}
        crop: {from: Crop, type: string?}
        size: {from: "Size (m2)", type: number}
    - from: ./workdays.yaml
      as: workdays
      schema: {date: date, jobs: array}
    - from: ./club/about.comp.md
      as: club
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Work days

::: each {in=(md.data.workdays) as=day}

**:value[Saturday, November 7, 2026]{of=(md.date(day.date, "full"))}**, from nine to noon: :value[turn the compost, mend the gate by plot 4, and plant the garlic]{of=(md.list(day.jobs))}.

::: /each

## The plots

::: each {in=(md.data.plots.filter(p, p.gardener != null)) as=p join=list}

- **Plot {{p.number}}**, {{p.size}} m²: {{p.gardener}}, growing {{p.crop}}

::: /each

:value[2 plots are free]{of=(string(size(md.data.plots.filter(p, p.gardener == null))) + " " + md.plural(size(md.data.plots.filter(p, p.gardener == null)), "plot is", "plots are") + " free")} for next year. A small plot is :value[$25.00]{of=(md.currency(md.data.club.fees.small, "USD"))} a year, a large one :value[$40.00]{of=(md.currency(md.data.club.fees.large, "USD"))}. Call {{md.data.club.contact}} at {{md.data.club.contact.phone}} to take one.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/data) in the playground.

The plots, as a spreadsheet saves them:

```csv snippet=data/plots.csv
Plot,Gardener,Crop,Size (m2)
1,Maya Okafor,garlic,12
2,Tomás Reyes,kale,12
3,,,20
4,Priya Nair,leeks,20
5,Jun Park,beans,12
6,,,12
```

The work days:

```yaml snippet=data/workdays.yaml
- date: 2026-11-07
  jobs: [turn the compost, mend the gate by plot 4, plant the garlic]
- date: 2026-11-21
  jobs: [clear the bean poles, cover the empty beds]
```

And the club's page, a CompMD document of its own in the folder `club/`, whose frontmatter holds the fees and the contact:

```md snippet=data/club/about.comp.md
---
md:
  version: 1
name: Elm Street Garden
contact: {name: Maya Okafor, phone: 555-0142}
fees: {small: 25, large: 40}
---

# {{md.doc.name}}

Six plots behind the library on Elm Street, kept by the neighbors who grow in them. Call {{md.doc.contact}} at {{md.doc.contact.phone}}.
```

The page lists both work days with their jobs, the four plots with a gardener, and *2 plots are free for next year*. When a plot changes hands, you change a row of the spreadsheet; the newsletter follows.

## Loading a file

A document lists the files it reads under `md.data`, one entry each:

- **`from`** is where the file is: `./plots.csv` is beside the document, `./club/about.comp.md` in the folder `club` beside it.
- **`as`** is the name the data gets: `as: plots` makes it `md.data.plots`.

From then on, `md.data.plots` works wherever `md.doc` does: in `{{…}}`, in expressions, in `each`. A list of records is a list of records, wherever it came from: chapter 5's `filter`, `exists` and `md.loop` work on it as they did on the frontmatter's.

## The formats

The file's extension says how it is read:

| File | Gives |
| --- | --- |
| `.csv` | a list of records, one per row, named by the header row |
| `.tsv` | the same, with tabs between the cells |
| `.yaml`, `.yml` | what the YAML holds, as in the frontmatter: a list, a record, a value |
| `.json` | what the JSON holds |
| `.comp.md` | the document's frontmatter keys, its `md.doc`; not its text, and not its own `md` settings |

YAML in a data file follows the frontmatter's rules (chapter 4): `2026-11-07` is a date written as text, and `no` is text, not false.

A CompMD document is a good home for values several documents share, such as the club's fees: it reads as a page of its own, and every issue that loads it gets the same values. `{{md.data.club.contact}}` is *Maya Okafor*, since a record with a `name` reads as its name.

## Spreadsheets: CSV

Save a sheet as CSV, and its first row becomes the names. Each row after it is a record, its cells under those names. Without anything else, every cell is text, and an empty cell is empty text, `""`. That is often enough to show a cell: `{{p.Gardener}}`.

But text doesn't count. `p.Plot + 1` is a diagnostic (`F024`), since `"1"` is text, not the number 1. A *schema* says what each column holds.

### A schema

`schema` lists the columns, each with its type, as you'd declare a param (Part IV uses the same types):

| Type | A cell | Read as |
| --- | --- | --- |
| `string` | `Maya Okafor` | text |
| `number` | `12`, `4.5` | a number, with a point for the decimals |
| `boolean` | `true`, `false` | true or false |
| `date` | `2026-11-07` | a date, year, month and day |

A YAML or JSON file's values already have their types, and `array` and `object` take its lists and records: `jobs: array` above. A schema there checks the records rather than reading them.

Write a schema in one line when the names fit: `schema: {date: date, jobs: array}`. Columns the schema doesn't name stay in each record, as text.

### Empty cells

A cell may be empty, as plot 3's gardener is. Add `?` to its type to allow that: `type: string?`, and an empty cell is `null`, no value. That's how the newsletter tells free plots from taken ones: `p.gardener != null`.

Without the `?`, an empty cell is a mistake in the row, as below.

### Names with spaces

A header such as `Size (m2)` can't be written in a path. Give the column a name of your own, with the header as its `from`:

```yaml
schema:
  size: {from: "Size (m2)", type: number}
```

The data then has `size`, and `{{p.size}}` works. A name is one word, or words joined by `_`, such as `plot_size`.

### Other separators

Some spreadsheets save CSV with a semicolon between the cells. Say so with `delimiter`:

```yaml
md:
  data:
    - from: ./plots.csv
      as: plots
      delimiter: ";"
```

Such files often write decimals with a comma, `1,5`, which isn't a number in a schema: declare the column as `string`, or save it with points.

## When something is wrong

Data comes from outside the document, so the document still shows when it is wrong:

| What | The diagnostic | What the page gets |
| --- | --- | --- |
| The file isn't there, or isn't valid CSV, YAML or JSON | `F026`, with where in the file when it can tell | `md.data.<name>` is `null`, and an `each` over it shows its `:: #empty` slot |
| A header with an empty or repeated name | `F026` | `null` |
| A row with more or fewer cells than the header | `F027`, at its line | the row is left out |
| A cell that doesn't fit its type, such as `x` for a `number`, or an empty cell without `?` | `F027`, naming the record and the column: *record 2: \`Plot\` isn't a number: \`x\`; it is dropped* | the row is left out |

A row left out is left out of every list made from the data, so a total over a list with an `F027` is short until the row is fixed. The diagnostic says which row.

## Where files may be

A path is relative to the document that writes it, as a link is: `./plots.csv` is beside it, `./club/about.comp.md` in a folder below. A path can't go up out of the document's project, the folder the app gives it: `../../other.csv` there is `F026`.

Values inside data are never paths: a `logo: ./logo.png` in the club's frontmatter, written into this document as `![]({{md.data.club.logo}})`, is a path from this document, not from the club's folder.

A pack (chapter 16) can bring data files too. A document that imports the pack reads one by the pack's name: `from: "@acme/garden-kit/crops.json"`.

## Data from a website

`from` can be an `https://` address:

```yaml
md:
  data:
    - from: https://weather.example.org/elm-street/frost.json
      as: frost
      fallback: ./frost.json
```

The document can't decide on its own to fetch from a site. The app that shows it decides which sites its documents may read from. A site it doesn't allow is `F028`, and the document gets its `fallback`, a file of its own folder, or `null` without one. A fetch that fails also takes the fallback. The playground fetches from no site, so there such a document shows `F028` and its fallback.

So give remote data a `fallback`, kept up to date now and then, and the page reads well wherever it's opened. The site you fetch from learns that the document was opened, and the values it asks with (chapter 7). A document carries no passwords: data that needs one comes through the app, below.

## Data from the app

An app can give its documents its own data, such as a calendar app its events. Its documents read it with `from: host:`, and what follows the colon is the app's to define:

```yaml
md:
  data:
    - from: host:events?calendar=garden
      as: events
      schema: {title: string, date: date}
      fallback: ./events.json
```

It lands in `md.data` like any file, with the same `schema` and `fallback`. Give it a schema, since two apps may give the same name different data. In an app that has no such data, the import is `F026` and takes its fallback, so a document that names `host:` works fully only in its own app. The playground has no `host:` data.

The [spec's data imports](https://compmd.hakanserce.com/spec/#86-data-imports) have every rule.

Next: [inputs the reader changes](07-inputs-the-reader-changes.md), for values the reader picks on the page: a plot size, a name, a month.
