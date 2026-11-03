# 1. A first document

A CompMD document is a Markdown file with a little more in it: values from its own header, and components, such as a box around a paragraph. In this chapter you write one, see it as a page, and see what it looks like where CompMD isn't known.

The book follows one document as it grows: the newsletter of a community garden. Each chapter adds to it, and each version opens in the playground, where you can change it and watch the page.

## Where to write

Anywhere you write Markdown: a text editor, a notes app, a repository on GitHub. A CompMD file ends in `.comp.md`, so tools know what it is.

To see the page as you type, use the [playground](https://compmd.hakanserce.com/playground/): the text on the left, the page on the right. Its *Learn CompMD* tour is a quick, hands-on introduction to everything this book covers.

## The document

Save this as `garden.comp.md`:

```md snippet=first-document/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: box {padding=m border=subtle border-radius=m}

**Work day:** Saturday, November 7, from nine to noon. Bring gloves; we bring the tea.

::: /box

The compost bins are full again, so the next load goes to the bins by the shed.
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/first-document) in the playground.

The page shows the heading *Elm Street Garden*, the line *News from the plots, November 2026.*, the work day in a box with a thin border and rounded corners, and the paragraph about compost.

## Its parts

**The frontmatter** is the part between the two `---` lines at the top. It is YAML: a key, a colon, and a value, one per line. Here it says three things:

- `md:` holds CompMD's own settings. `version: 1` says which version of CompMD the document is written in. With the `md` key, the file stays CompMD wherever it goes, even renamed to `.md`.
- `title` and `issue` are yours. Any key that isn't `md` is the author's, and you can name it what you like.

`version` is one of a few settings under `md`. The others come up in the chapters ahead:

| Under `md` | Says | Chapter |
| --- | --- | --- |
| `version` | the version of CompMD, `1` | this one |
| `lang` | the document's language, such as `en` or `de`, for dates and numbers | [4](04-values-and-expressions.md) |
| `dir` | the direction the text runs, `ltr`, `rtl` or `auto`; by default the direction of `lang`'s script | [3](03-containers-and-params.md) |
| `tz` | the time zone of "today", such as `Europe/Berlin` | [8](08-dates-numbers-and-lists.md) |
| `syntax` | extra syntax the document turns on, such as wiki links | [2](02-the-markdown-you-know.md) |
| `data` | files and addresses the document reads values from | [6](06-data-from-files.md) |
| `inputs` | values the reader changes | [7](07-inputs-the-reader-changes.md) |
| `import` | the packs the document uses | [16](16-using-a-pack.md) |

A key under `md` that CompMD doesn't know is a warning that names the nearest one it does, so a misspelt `lnag` doesn't go unnoticed.

**`{{md.doc.title}}`** puts the value of your `title` key where it stands. `md.doc` is your frontmatter, so `{{md.doc.issue}}` is *November 2026*. Change the value at the top, and every place that uses it follows. Chapter 4 says more about values.

**The box** is a *component*: a piece of the page with a name, here `box`. It starts with a line of its own, `::: box`, and ends with `::: /box`. What's between the two lines is its body, ordinary Markdown; the blank lines around it are there for plain renderers, as the next section shows. What's in the braces, `{padding=m border=subtle border-radius=m}`, are its *params*: a medium padding, a subtle border and medium rounded corners. Chapter 3 is about components and params.

**Everything else** is Markdown, as you know it: a heading, paragraphs, bold text. Chapter 2 says what Markdown you can use.

## Where CompMD isn't known

A CompMD file is a Markdown file, so it reads well where nobody knows CompMD: on GitHub, in an editor's preview, in any Markdown viewer. Such a *plain renderer* shows the heading, the paragraphs and the bold text as usual, and the rest as you wrote it. The box's lines are short paragraphs of their own, around the work day:

```text
::: box {padding=m border=subtle border-radius=m}

Work day: Saturday, November 7, from nine to noon. Bring gloves; we bring the tea.

::: /box
```

They are paragraphs of their own because of the blank lines around them. Without those, a plain renderer would run the box's lines and the work day together into one paragraph. So leave a blank line before and after every line that starts with colons; this book always does.

The heading reads `{{md.doc.title}}`, which tells a reader where the title comes from. Nothing is lost, and nothing turns into noise. This matters as you write: a document should still make sense as text. The chapters ahead point out how each part of CompMD keeps it so.

> [!TIP]
> In the playground, the **Plain Markdown** tab beside the page shows what a plain renderer shows.

## When something is wrong

Mistakes don't stop the page. Delete the `::: /box` line, and the page still renders, with the box running to the end of the document. Under the text, the playground lists a *diagnostic*: what is wrong, where, and often a button that fixes it. Here it says that the box was never closed.

Each diagnostic has a code, such as `F004`, so you can look it up. Codes starting with `F` are failures, which change what renders; codes starting with `W` are warnings, which don't.

A diagnostic also says where it is, as a line and a column, and what is wrong, in a sentence. The one for the missing closer reads:

```text
F004 12:1  `::: box` isn't closed
```

Line 12, column 1 is where the box opens, counting the frontmatter's lines too; the playground marks the place in the text, so you needn't count. A mistake gives one diagnostic, not a cascade of them: the box that runs on doesn't add a diagnostic for everything after it.

Whatever goes wrong, CompMD shows your text rather than hide it: a component it can't show is replaced by its fallback (chapter 3), never by a blank. [Chapter 20](20-the-diagnostics-and-their-fixes.md) lists every code with its fix.

Next: [the Markdown you know](02-the-markdown-you-know.md), and what CompMD adds to it.
