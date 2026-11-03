# 2. The Markdown you know

Everything you write in Markdown works in CompMD, as GitHub shows it. This chapter is a tour of it: what you already know, what GitHub added, and the few things CompMD adds before any component. The newsletter grows a notice, a work day's list, a table of plots and a formula.

## The newsletter, in Markdown

```md snippet=markdown/garden.comp.md
---
md:
  version: 1
  syntax: [wiki-links]
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

> [!IMPORTANT]
> The water goes off on November 15 for the winter. Empty your cans and hoses before then.

## Work day

Saturday, November 7, from nine to noon.[^tea] On the list:

- [x] Order wood chips
- [ ] Turn the compost
- [ ] Mend the gate by plot 4
- [ ] Plant the garlic

[^tea]: Tea and soup from eleven, by the shed.

## Plots for next year

| Size | Plots | Fee for the year |
| --- | ---: | ---: |
| Small, 2 × 3 m | 12 | $25 |
| Large, 3 × 4 m | 8 | $40 |

A large plot is $3 \times 4 = 12$ square meters, twice a small one. Compost heats well with about thirty parts brown to one part green:

$$
\frac{\text{brown}}{\text{green}} \approx 30
$$

More in [[Compost]], and the sign-up rules in [[Plots|the plot rules]].

<details>
<summary>Who to ask</summary>

Maya keeps the keys, and Tomás the tools.

</details>
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/markdown) in the playground.

## Markdown

Headings, paragraphs, *emphasis*, **strong text**, `code`, links, images, quotes, lists, code blocks and lines between sections all work as in [CommonMark](https://commonmark.org), the standard most Markdown tools follow. If you know Markdown, you know this part of CompMD.

## What GitHub added

GitHub's Markdown adds a few things, and CompMD has them all:

- **Tables**, with a line of dashes under the header. A colon on the right of the dashes aligns the column to the right, as the numbers above; on the left, to the left; on both sides, in the center.
- **Task lists**: `- [ ]` for a box to tick, `- [x]` for one ticked.
- **Strikethrough**: `~~like this~~`.
- **Links without brackets**: a web address such as `https://example.com` becomes a link by itself.
- **Footnotes**: `[^tea]` where the note is referred to, and `[^tea]: …` anywhere for its text. Notes gather at the end of the page, numbered.
- **Alerts**: a quote whose first line is `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]` or `[!CAUTION]` becomes a notice of that kind, with its own color and icon.

### Tables, closer

A table needs its header row and the line of dashes under it; the rows after them are optional. Each row is one line, with its cells between `|` signs. A row with fewer cells than the header gets empty ones, and cells past the header's count are dropped.

A cell holds one line of inline Markdown: emphasis, links, code, images and values (`{{…}}`, chapter 4), but no list and no second paragraph. A `|` that is part of a cell's text is written `\|`, even inside code or a wiki link, because the table splits its rows into cells first:

```md
| Tool | Shortcut |
| --- | --- |
| Pipe | `a \| b` |
```

### Footnotes, closer

A footnote's label is yours, a word or a number, and only links the reference to its note: the page numbers the notes in the order they are referred to. A note's text can go anywhere in the document, and a reference with no note is shown as written.

## Code blocks

A code block starts and ends with a line of three backticks, and the word after the first three names its language, so the page can color it: ` ```yaml `. What's inside always shows as written, values and colons included. Some languages draw something instead, such as a diagram, when the document uses a pack that draws them ([chapter 17](17-code-blocks-a-pack-draws.md)); without one, they stay code.

## Math

Math is written in TeX, the language of scientific papers. Inline, it goes between single dollar signs: `$3 \times 4 = 12$`. On a line of its own, it goes between two lines of `$$`, or in a code block whose language is `math`, as GitHub also reads it:

````md
```math
\text{area} = \text{width} \times \text{length}
```
````

The page typesets it; a plain renderer shows the TeX.

A dollar sign that is money is just text. `$25` is not math, because math needs a dollar sign at both ends, and the rule for the closing one keeps prices out: it can't be followed by a digit or come after a space. If a sentence still reads as math by mistake, write the dollar sign as `\$`.

## Wiki links

`[[Compost]]` links to a page by its name, as wikis and many notes apps write it, and `[[Plots|the plot rules]]` shows *the plot rules* for the page *Plots*. They are an option, because in plain Markdown the same brackets mean something else, so a document turns them on in its frontmatter:

```yaml
md:
  syntax: [wiki-links]
```

Without that line, `[[Compost]]` stays as written, brackets and all. A name under `syntax` that CompMD doesn't know is a warning; `wiki-links` is the only one in version 1.

Where a wiki link leads is up to the app that shows the document. Where there are no pages to link to, as in the playground, the link's text shows as text.

## HTML

Markdown lets you write HTML, and CompMD shows a short list of safe tags, as GitHub does, because a document may come from anyone:

| Tag | For | Keeps |
| --- | --- | --- |
| `<details>`, `<summary>` | a part the reader opens, as above | `open`, to show it opened |
| `<sub>`, `<sup>` | H<sub>2</sub>O, 10<sup>3</sup> | |
| `<kbd>` | a key to press: <kbd>Ctrl</kbd> | |
| `<br>` | a line break inside a paragraph | |
| `<img>` | an image | `src` and `alt` |
| `<a>` | a link | `href` |

Any other attribute, such as `style` or `class`, is dropped, and so is an address that isn't a web address, an email address (`mailto:`) or a relative one. Any other tag shows as text, as written, and so does a tag that is opened and never closed. A comment, `<!-- like this -->`, doesn't show at all, on the page or on GitHub, so it can hold a note to yourself.

CompMD reads nothing inside HTML. A line that starts with a tag begins an *HTML block*, which runs to the next blank line: `{{md.doc.title}}` in it shows as written, and so does a line of colons, which is a warning (`W116`) since it looks like a component and isn't one. So leave a blank line after `<details>` and before `</details>`, as above: the lines between are Markdown again. A tag inside a paragraph, such as `<kbd>`, ends where it closes, and the text around it is Markdown as usual.

Prefer Markdown and components where they do the job: they read better as text, and they look like the rest of the page.

## Links to a passage

A link can point at a passage of the page by its words: `[the water notice](#:~:text=The%20water%20goes%20off)` scrolls to *The water goes off* and marks it. The address is the web's own form for this; each space is written `%20`. In the playground, such a link selects the words in the text instead, so a document can point at what to change.

After `#:~:text=` come the words, in up to four parts separated by commas:

| Written | Marks |
| --- | --- |
| `#:~:text=Empty%20your%20cans` | the first place these words appear |
| `#:~:text=The%20water,the%20winter` | from *The water* to the next *the winter*: a longer passage, without writing it all |
| `#:~:text=Work-,day` | *day* where it comes right after *Work*, when the same words appear more than once |
| `#:~:text=nine,-to%20noon` | *nine* where *to noon* comes right after it |

Since a comma and a dash have a meaning here, one that is part of the words is written `%2C` or `%2D`, and an `&` is `%26`. A link to a passage is an ordinary link, so a plain renderer shows it as one, and a browser that doesn't know the form opens the page at its top.

## Writing colons and braces as text

CompMD gives a meaning to lines that start with colons, to a colon right before a word and brackets (`:value[…]`), and to double braces (`{{…}}`). When you mean them as text, put a backslash before them: `\::: not a box`, `\{{not a value}}`. The backslash doesn't show. Inside code, nothing needs it: `` `{{title}}` `` shows as written.

A time such as 10:30 or a word such as `std::vec` needs nothing either: a colon only starts a component after a space, at the start of a line, or after punctuation.

## Plain Markdown is CompMD too

A document without a single component is still a CompMD document, and gets what CompMD gives every document: the page as you type, diagnostics, and values from its frontmatter. A file ending in `.md` without an `md` key in its frontmatter stays ordinary Markdown, so CompMD never changes a file you didn't mean for it.

Next: [containers and params](03-containers-and-params.md), the components that make a page out of a text.
