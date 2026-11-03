# 20. The diagnostics and their fixes

When something in a document isn't right, CompMD says so with a *diagnostic*: a code, the place, and a message that says what it found. This appendix lists the diagnostics a writer meets, grouped by what you were doing when it came up, each with what it means and how to fix it. The [spec's catalogue](https://compmd.hakanserce.com/spec/#102-catalog) has every code, and the [Developer's Guide's appendix](../developers/21-the-diagnostics.md) the ones only an app or a pack's author causes.

## Reading a diagnostic

- **`F` codes are errors:** the document says something CompMD can't do as written. **`W` codes are warnings:** the document works, but something is probably not what you meant.
- **The page still shows.** In the playground and in editors, CompMD drops the smallest broken piece, a param, a line or a record, shows the rest, and shows a component's fallback where it can't show the component. A tool that publishes may refuse a document with errors, so fix them all.
- **One mistake gives one diagnostic.** A param that is refused doesn't also give a diagnostic at each place it would have been used.
- **Some come with a fix.** For a misspelt name, the message says *did you mean …?*, and the playground offers a button, **Use** and the right name, that makes the change. Those are marked *Fix offered* below.

## In the frontmatter

| Code | Means | Fix |
| --- | --- | --- |
| `F001` | `md.version` names a version this app doesn't read, such as `2`; the document is shown as plain Markdown | write `version: 1`, or leave it out |
| `W110` | `md.version` is a later minor version, such as `"1.3"`, than the app reads; what it doesn't know gives its own diagnostics | write the version the app reads, or use an app that reads the later one |
| `W111` | a key under `md` that CompMD doesn't know, such as `imports`, is ignored; the message names the nearest one. Also a name in `md.syntax` it doesn't know | correct the key: `import`. A misspelt key under `md` is a key that does nothing, so what it held, such as the inputs, is missing too |
| `F033` | the frontmatter isn't YAML as CompMD reads it: a mistake in the YAML, a key given twice, or an anchor (`&a`), alias (`*a`) or tag (`!!str`); the document is shown as plain Markdown | fix the YAML at the place the message gives; write a repeated value out again instead of an anchor |

## Writing components

| Code | Means | Fix |
| --- | --- | --- |
| `F016` | a line starts with `::` but is no component line: `:::box` (no space), `:: /box` (two colons for a closer), `::::` (four colons), or `::: @who/box` (a pack's name in the text); the line is shown as text | write the form the message gives: `::: box`, `::: /box`, `:: name[…]`, `:: #name`, `::: + …`. To write the line as text, start it with `\` |
| `F003` | a closer names no open container, such as `::: /bxo` for `::: box`; it closes the innermost one, which the message names | correct the closer's name |
| `F004` | a container isn't closed before the end of the document, or before the closer of a container around it; it runs to there | add its closer, `::: /name`, where its body ends |
| `F005` | a closer while no container is open; it is ignored | remove it, or add the opener it belongs to |
| `F032` | a closer in a list item or quote names a container opened outside it; it is ignored | put the closer at the same level as its opener, outside the list item or quote |
| `F012` | a `::: +` line that doesn't follow an opener, a leaf or another `::: +` line, often because of a blank line between them; it is ignored | move it directly under the line it adds to |
| `F015` | a slot line, `:: #name`, outside any container; it is ignored | move it into the container, or remove it |
| `F007` | a container has the same named slot twice; the second one's content joins the first | merge the two parts under one slot line |
| `F018` | a component that takes no body is written as a container, such as `::: mermaid` … `::: /mermaid`; the body shows after it, as ordinary content | write it as a leaf, `:: mermaid[…] {…}` |
| `W108` | a component line directly after a line of a paragraph; a plain Markdown viewer reads it as part of the paragraph | put a blank line before it |
| `W116` | a line starting with `::` inside an HTML block, which makes it part of the HTML | put a blank line between the HTML and the component line |
| `W107` | an inline component followed by a space, then braces: `:value[x] {of=(…)}`; the braces are shown as text | remove the space: `:value[x]{of=(…)}` |
| `W101` | a leaf without fallback text, `:: chart {…}`, which shows nothing in a plain viewer and has no text for screen readers; or a code block a pack draws without `alt` | write what it shows in the brackets, `:: chart[Sign-ups rose to 31] {…}`, or add `alt="…"` after the block's language (chapter 17) |

## Writing params

| Code | Means | Fix |
| --- | --- | --- |
| `F008` | a param the component doesn't take, such as `colour=red`; it is ignored. Also a breakpoint that isn't `sm`, `md` or `lg`, or a breakpoint on a param that takes none, such as `md:show` | *Fix offered* for a misspelt param: `pading` becomes `padding`. Otherwise remove it; a component's signature, or its pack's README, says which params it takes |
| `F009` | a value the param doesn't take, such as `border=subtel`, or `count=many` for a number; the param is treated as absent, so its default is used | *Fix offered* for a misspelt word or colour: `subtle`. Otherwise write a value of the kind the message names |
| `F010` | a required param is left out; the component shows its fallback | add it, as the message names it |
| `F011` | a param list CompMD can't read: a quote or bracket that doesn't close, a value with `(`, `{` or a space that isn't quoted, such as `grid-template-columns=repeat(3, 1fr)`, or `=` with nothing after it. On a block line, the params before it are kept; in an inline component, none are, and the list is shown as text | close the quote or bracket; quote the value, `"repeat(3, 1fr)"`, or make it an expression, `title=(md.doc.title)`; write `title=""` for empty text |
| `F017` | a param given twice, counting `::: +` lines, or `#name` and `id=` together, or `code` in the braces of a code block a pack draws; the first is used | remove the second |
| `F006` | a slot line the component doesn't declare, such as `:: #els` in an `if`; its content joins the default slot | *Fix offered*: `els` becomes `else` |
| `W114` | a box whose blocks change at a breakpoint gets its width from its content, in a row for example, and so has none | give it a `width` (chapter 10) |
| `W119` | a box that places blocks by position (`inset-*`) has no height to place them in, at some width | give it a `height`, an `aspect-ratio` or `fill` (chapter 11) |
| `W120` | a param for a box given to a component that has no box of its own (one defined `display=contents`); it is ignored | remove it, or put it on a `box` around the component |

## Values and expressions

| Code | Means | Fix |
| --- | --- | --- |
| `F014` | a path in `{{…}}` that doesn't exist, such as `{{md.doc.isue}}`; it is shown as written. Also prose that mentions double braces | correct the path; to write the braces as text, write `\{{`, or put them in a code span |
| `W106` | `{{` followed by something that isn't a path, such as `{{ 1 + 2 }}`; it is shown as text | for a computed value, use `:value[…]{of=(…)}` (chapter 4); for the braces as text, `\{{` |
| `F024` | a mistake in an expression: a misspelt name, a value of the wrong type (`"Plot " + 4`), or one that fails when computed; the message points to the place in the expression. Also a list, or a record without a `name`, put in `{{…}}` | correct the expression: `"Plot " + string(4)`; write a list with `md.list(…)` or `each` (chapter 5) |
| `W109` | a link or image address made with `{{…}}` that is neither a relative address nor one starting with `https:`, `http:` or `mailto:`; the text is shown without the link | check the value: an address from the frontmatter or data should start with `https://` |
| `F013` | a name declared twice in the document; or `if`, `each`, `value`, `box` or `md` defined, imported or renamed to; or a word expressions keep for themselves (`in`, `as`, `list`, `string`, …) as an `as` name, an input or a definition's param; or two params whose names differ only in `-` and `_`. The first declaration wins | choose another name: `as=job`, not `as=in` |
| `F025` | an `each` whose body doesn't have the shape its `join` needs: list items for `join=list`, one paragraph for `join=inline`; the items are added one after another | make the body one list item, or one paragraph, or remove `join` |
| `W103` | an `each` whose body is one list item, without `join`; each item becomes a list of its own | add `join=list` |
| `F034` | the page would have more than the app allows, at least 100,000 parts, usually from `each` inside `each` inside `each`; the `each` that crossed the limit shows its body once | filter the lists first, or show fewer levels |

## Data and inputs

| Code | Means | Fix |
| --- | --- | --- |
| `F026` | data that can't be loaded: a file that isn't there, an address that doesn't answer, a file CompMD can't read, a `host:` source the app doesn't give, a path that leaves the document's folder (`../`), or a CSV header with an empty or repeated name. The data is its `fallback` file if it has one, otherwise empty, so an `each` over it shows its `empty` slot | correct the path or the file at the place the message gives; give a remote source a `fallback` |
| `F027` | a record that doesn't match the `schema`, such as `twelve` where a number is declared, or a row of a CSV file with more or fewer cells than the header; the record is dropped | correct the record, or the schema's type; declare a column that may be empty as optional: `number?` |
| `F028` | data from an address the app isn't set up to fetch from; the data is its `fallback`, or empty | ask the app's owner to allow that address, or give the entry a `fallback`. An app that asks its readers shows the data once the reader allows it |
| `F031` | a `from` fills in something it may not: anything but `md.input` and `md.doc`, a value in the address's `https://` and host, or an input a component in code can change | fill in only the path or query of the address, from inputs and the frontmatter |
| `F030` | an input component's `bind` names an input that isn't declared under `md.inputs`, or one of a type it can't edit, such as a `number-input` bound to a choice of words | declare the input, correct its name, or use the input component for its type (chapter 7) |

## Definitions and components of your own

| Code | Means | Fix |
| --- | --- | --- |
| `F002` | a component name the document doesn't know, such as `:: bxo`; it shows its fallback. Also a signature fence with no component of its name to attach to | *Fix offered* for a misspelt name: `box`. Otherwise import the component, or define it |
| `W112` | a definition or import with the name of a built-in component, such as `row`, which this document's `row` now means | keep it, if that is what you want, or choose another name |
| `F019` | a definition that uses itself, directly or through others, such as a `card` whose body has a `card`; the use that closes the circle shows its fallback | remove the circle: a definition can't contain itself |
| `F036` | a signature fence with params in braces after it; they are ignored | put them on the component's own fence, which styles its box |
| `F037` | a type in a signature, an input or a schema that CompMD doesn't have, such as `numbr`; that param takes any value | *Fix offered*: `number` |
| `W105` | a component in code, in a fence, without a signature fence; its params go through unchecked | add a signature fence (chapter 18) |
| `W102` | a component in code that this app can't run: a `runtime` it doesn't have, such as `react@18`, any `runtime` on a `js` or `ts` fence, or an app that makes no web page; it shows its fallback | use a runtime the app has, or leave `runtime` out |
| `F029` | a component in code that the document exports, without a signature fence, which the documents that import it need, since they may be shown where its code can't run | add a signature fence |
| `W117` | a component in code that the document exports, without `runtime=` | add `runtime=`: `react@19`, `preact@10` or `svelte@5` |

## Packs and imports

| Code | Means | Fix |
| --- | --- | --- |
| `F026` | an import that can't be had: no pack of that name, no version that matches, a pack the app doesn't allow or that is too large, a file that isn't there, or a `from` that isn't a pack's name (`@who/what`) or a path. The import is dropped, and its components show their fallbacks | correct the name or the version (chapter 16); the message says which of these it is |
| `F021` | an imported name its source doesn't offer; a file of code offers one component, named after the file | correct the name, or see the pack's README for what it offers |
| `F020` | documents that import each other in a circle; the import that closes it is dropped | remove one of the imports |
| `F023` | a name two `names: "*"` imports both give, or one of them and a built-in component, is used; or a language two code renderers both draw. It shows its fallback | list the name you mean in one import's `names` |
| `F009` | `integrity` on an import whose version isn't exact, such as `@0`; it is ignored | pin an exact version, `@0.1.1`, or remove `integrity` |
| `F035` | the pack doesn't match the fingerprint in `integrity`; the import is dropped | check the version and its fingerprint; a tool should write it (chapter 16) |
| `W113` | a component from a pack the app itself provides, which other apps don't have; there, it shows its fallback | nothing, for a document only that app shows; for others, import a published pack instead |
| `W118` | a component in code failed to draw what it was given, such as a diagram Mermaid can't read; a leaf shows its fallback, and a code block its code | correct its params, or the block's text; the message says what the component found |
| `W121` | a component in code can't load: it needs a library or framework the app doesn't have, or its code doesn't compile; it shows its fallback | use the component in an app that has what it needs, or tell the component's author |

[Back to the contents](README.md)
