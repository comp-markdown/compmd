# 19. The syntax at a glance

Everything a writer types in CompMD, on a few pages: the frontmatter, the three forms of a component, params, slots, values and expressions, `if` and `each`, definitions, imports and exports, and code blocks a pack draws. Each table links to the chapter that teaches it, and the [spec](https://compmd.hakanserce.com/spec/) has every rule.

## The frontmatter

Everything CompMD reads is under one key, `md`. Every other key is yours, read as `md.doc.<key>` (chapter 4).

| Key under `md` | Holds | Example |
| --- | --- | --- |
| `version` | CompMD's version; `1` when left out | `version: 1` |
| `import` | the components the document uses from packs and files (below) | `- from: "@comp-markdown/mermaid@0"` |
| `export` | the components the document offers others (below) | `names: [notice]` |
| `data` | data from files and addresses, read as `md.data.<as>` (chapter 6) | `- from: ./plots.csv` |
| `inputs` | values the reader changes, read as `md.input.<name>` (chapter 7) | `size: small \| large = small` |
| `lang` | the document's language, which formatting follows; `en` when left out | `lang: de` |
| `dir` | its text direction, `ltr`, `rtl` or `auto`; from `lang` when left out | `dir: rtl` |
| `tz` | the time zone of `md.today()` and `md.now()` | `tz: Europe/Berlin` |
| `syntax` | extra syntax the document asks for: `[wiki-links]` for `[[Page]]` | `syntax: [wiki-links]` |

**YAML to remember.** `true` and `false` are the only yes and no; a date such as `2026-11-07` is text, which date functions read; quote what would read as a number, `"1.10"`, or as something else, `"9:30"`. No anchors (`&a`), aliases (`*a`), tags (`!!str`) or repeated keys (`F033`).

### A data entry

| Key | Holds |
| --- | --- |
| `from` | `./file.csv`, `.tsv`, `.yaml`, `.yml`, `.json`, `.comp.md` (its frontmatter), a file in an imported pack (`@scope/pack/file`), an `https://` address, or `host:` data the app gives; may hold `{{md.input.…}}` and `{{md.doc.…}}` |
| `as` | its name: `as: plots` is `md.data.plots` |
| `schema` | the type of each field of a list of records: `{plot: number, gardener: string}`; `{from: "Plot no.", type: number}` renames a column |
| `cache` | for an address: how long a copy is kept, `30s`, `10m`, `1h`, `1d` |
| `fallback` | for an address or `host:`: a local file to use when it fails |
| `delimiter` | for a CSV file: the character between fields, `,` by default |

### An input

`<name>: <type> = <default>`, every input with a default: `start: date = (md.today())`, `size: small | large = small`, `show_all: boolean = false`. An input's name is one word, or words joined by `_`.

## Components

The number of colons tells the form (chapter 3):

| Form | Written | Notes |
| --- | --- | --- |
| Container | `::: name {params}`, a body, `::: /name` | the closer repeats the name; containers nest |
| Leaf | `:: name[fallback] {params}` | one line; the fallback is what shows where the component can't |
| Inline | `:name[fallback]{params}` | inside text; no space before `{`, and the brackets are needed |
| Continuation | `::: + params` | more params for the line above it, with no blank line between |
| Slot | `:: #name` | starts a named part of the container it is in |

- Put a blank line before and after a block component's lines, so plain viewers keep the text around them as you wrote it (`W108`).
- A backslash makes a line or an inline component text: `\::: box`, `\:value[x]`.
- Component lines work in list items and quotes, indented as the item's text is; not in code or HTML.

### Built in

| Component | Does | Chapter |
| --- | --- | --- |
| `box` | a box, with any of the common params | 3 |
| `stack`, `row`, `grid` | blocks top to bottom, side by side, or in a grid | 9 |
| `if`, `each` | shows a part when true; repeats a part for each item | 5 |
| `value` | computed text: `:value[fallback]{of=(…)}` | 4 |
| `date-input`, `number-input`, `text-input`, `select-input`, `toggle-input` | lets the reader change an input: `:: number-input[Plots: 4] {bind=plots}` | 7 |

## Params

| Written | Is |
| --- | --- |
| `title="Plot 4"` | text in quotes; may hold spaces, `{{…}}`, and `\"` for a quote |
| `padding=m` | a bare value: a word or number, read as the param's type says |
| `count=3` | the number 3 where the param takes a number, the text `3` where it takes text |
| `scroll` | a flag: `true` |
| `tags=["a","b"]`, `meta={"id":1}` | a list or a record, written as JSON on one line |
| `background=(soon ? "accent" : "subtle")` | an expression, computed (chapter 4) |
| `.notice` | a class (may repeat) |
| `#greeting` | the id, for links: `[the greeting](#greeting)` |
| `md:padding=l` | from the width `sm` (480 px), `md` (768 px) or `lg` (1024 px) up (chapter 10) |

- Params are separated by spaces. Each may be given once (`F017`), counting continuation lines.
- A bare value can't hold `(`, `)`, `[`, `]`, `{`, `}`, spaces or quotes: quote it, `"repeat(3, 1fr)"`, or make it an expression. `title=` with nothing after it is `F011`; `title=""` is empty text.
- `{{…}}` works in params only inside quotes: `title="{{md.doc.title}}"`, or `title=(md.doc.title)`.
- Every component takes `class`, `id`, `lang` and `dir`; every block takes the common params of its box (chapter 3): `width`, `height`, `min-width`, `max-width`, `padding` and its sides, `border`, `border-radius`, `background`, `grid-area`, `order`, `align-self`, `justify-self`, `aspect-ratio`, the `inset-*` positions, `scroll`, `fill`, `sticky`, `show` and `hide`.

## Slots and template slots

| Written | Is | Chapter |
| --- | --- | --- |
| content before the first slot line | the default slot | 13 |
| `:: #else`, `:: #footer` | a named slot, up to the next slot line or the closer; once per container | 13 |
| `{{md.slot}}`, `{{md.slot.footer}}` | in a definition, where a slot's content goes | 13 |
| `::: each {in=(…) as=job}` … `{{job}}` | a template slot: the body repeats once per item, the item named by `as` | 14 |

A slot line starts at the container's own level, never indented under a list item in it.

## Values and expressions

| Written | Is | Chapter |
| --- | --- | --- |
| `{{md.doc.issue}}` | a value from the frontmatter, inserted as text | 4 |
| `{{md.data.fees.small}}`, `{{md.input.size}}` | a value from data, and an input's value | 6, 7 |
| `{{md.lang}}`, `{{md.version}}`, `{{md.tz}}` | the document's language, version and time zone | 4 |
| `{{job}}`, `{{md.loop.number}}` | inside `each`: the item, and where the list is | 5 |
| `{{title}}` | inside a definition: a param | 12 |
| `\{{` | two braces as text | 4 |
| `:value[$25.00]{of=(md.currency(25, "USD"))}` | computed text, with a fallback | 4 |
| `(…)` in a param | an expression in CEL | 4 |

`{{…}}` takes a path only: names joined by dots, a param named with `-` written with `_`. It works in text, headings, table cells, link addresses and quoted params, never in code. A list, or a record without a `name`, can't be inserted (`F024`).

**In expressions:** `+ - * /`, `== != < <= > >=`, `&& || !`, `a ? b : c`, text in double quotes, `string(…)`, `int(…)`, `double(…)`, `size(…)`, `x in list`, and the list macros `filter`, `map`, `exists`, `all`, `exists_one` (chapter 5). For text: `contains`, `startsWith`, `endsWith`, `lowerAscii`, `upperAscii`, `replace`, `split`, `join`, `substring`, `trim`, `charAt`, `indexOf`, `lastIndexOf`.

| CompMD's functions | |
| --- | --- |
| Formatting | `md.date(d, style)`, `md.currency(n, code)`, `md.number(n, digits)`, `md.plural(n, one, other)`, `md.list(list)` |
| Dates | `md.today()`, `md.now()`, `md.addDays(d, n)`, `md.daysBetween(a, b)`, `md.startOf(d, unit)`, `md.dateOf(t)` |
| Lists | `md.sort`, `md.sortBy(list, field)`, `md.reverse`, `md.slice(list, start, end)`, `md.sum`, `md.min`, `md.max`, `md.unique`, `md.groupBy(list, field)` |

Chapter 8 has each one, with examples.

## `if` and `each`

| | Params | Slots |
| --- | --- | --- |
| `if` | `when`, true or false | the default, shown when true; `else`, shown when false |
| `each` | `in`, the list; `as`, the item's name; `join`: `none` (the default), `list` or `inline`; `sep` and `last`, between items with `join=inline` | the default, once per item; `before`, above the items when there are any; `empty`, when there are none |

`md.loop` inside `each`: `index` (from 0), `number` (from 1), `count`, `first`, `last`.

## Definitions

A component written in CompMD (chapter 12) is a fence: the language `compmd`, `component=` and its name, params for its box, a signature, `---`, and a body.

````text
```compmd component=notice {padding=m border-radius=m background=accent}
params:
  title: string
  tone: info | warn = info
slots: [default]
---
**{{title}}** {{md.slot}}
```
````

| In a signature | Is | Chapter |
| --- | --- | --- |
| `title: string` | a required param | 15 |
| `note: string?` | an optional one, with no value when left out | 15 |
| `tone: info \| warn = info` | a choice of words, with a default | 15 |
| `{type: object, default: {a: 1}, doc: …}` | the long form, for defaults that don't fit on a line | 15 |
| `slots: [default, footer]` | the slots it takes; without `slots`, it takes no body | 13 |
| `slots: {default: {each: rows, as: as}}` | a template slot over the param `rows`, its item named by the param `as` | 14 |
| `headings: absolute` | headings in the body keep their level, instead of fitting where it is used | 12 |

**Types:** `string`, `number`, `boolean`, `date`, `array`, `object`, `name` (a name the caller picks, as `as` is), the styling types `length`, `space`, `color`, `tracks`, `areas`, `surface`, `border` and `radius`, a choice `a | b | c`, and `input<type>` for a param bound to an input.

A component in code (chapter 18) is a fence in its language, `jsx`, `tsx`, `svelte`, `js` or `ts`, with `component=` and an optional `runtime=` (`react@19`, `preact@10`, `svelte@5`), and its signature is a fence of its own: ` ```compmd component=harvest signature `.

## Imports and exports

````text
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
    - from: ./notices.comp.md
      names:
        - name: notice
          as: alert
    - from: ./harvest.jsx
      names: [harvest]
  export:
    pack: "@elm-street/garden"
    version: "1.0.0"
    names: [notice]
````

| Key | Holds | Chapter |
| --- | --- | --- |
| `from` | a pack, `@scope/name@version`, or a file: `./x.comp.md`, `.jsx`, `.tsx`, `.svelte`, `.js`, `.ts` | 16, 18 |
| the version | `@1`: the newest 1.x; `@^0.1.1`: 0.1.1 or a newer 0.1; `@0.1.1`: exactly that | 16 |
| `integrity` | the pack's fingerprint, with an exact version: `sha512-…` | 16 |
| `names` | the components used: `[a, b]`, `{name: a, as: b}` to rename, or `"*"` for all | 16 |
| `export.names` | the components offered: their names, `{name, as}`, or `"*"` for every definition | 12 |
| `export.pack`, `export.version` | the name and version a published pack has | 12 |

`if`, `each`, `value` and `box` can't be defined, imported or renamed to (`F013`).

## Code blocks a pack draws

| Written | Is |
| --- | --- |
| ` ```mermaid ` … ` ``` ` | drawn by an imported code renderer for `mermaid`; otherwise a code block |
| ` ```mermaid {theme=dark alt="…"} ` | params after the language, in braces; `alt` is the text for screen readers |
| `:: mermaid[fallback] {code=(…)}` | the same component as a leaf, for a diagram made from values |

A code block's text is never filled in from values. Chapter 17 has the rest.

Next: [the diagnostics and their fixes](20-the-diagnostics-and-their-fixes.md).
