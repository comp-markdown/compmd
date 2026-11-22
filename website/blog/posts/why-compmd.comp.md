---
title: Why CompMD
date: 2026-11-22
summary: Markdown is where I write everything, until a page needs more than text. Why I didn't want MDX, raw HTML or one app's shortcodes, and what CompMD gives a writer while keeping the file plain.
tags: [about, design]
---

Remember when a Markdown file was just text? You wrote a heading, a few paragraphs, a list, and it looked fine everywhere:
on GitHub, in your editor, in the notes app on your phone. Life was simple.

Then the page wanted more. Two columns. A note pinned to the side. A table that comes from a spreadsheet you update
every week, not one you copy by hand. A diagram. And Markdown, which is wonderful at text, has nothing for any of it.

I hit that wall again and again, with notes, newsletters and documentation alike. CompMD is what I built so I could
stop hitting it.

## What are our options?

When Markdown runs out, there are three usual ways out. I have used all three, and each one costs something.

- **Raw HTML in the Markdown.** It works, until you open the file somewhere else. A `<div class="two-col">` means
  nothing to GitHub, and a page full of them stops being something you'd call Markdown.
- **MDX.** Components in Markdown, written in JSX. Powerful, but the file is now a program: it imports code, it runs
  code, and outside a React build it reads as a mix of Markdown and JavaScript. Opening a post someone sent you means
  trusting their code.
- **One app's shortcodes.** Every static site generator and notes app has its own: `{{< figure >}}`, `[[embed]]`,
  `{% include %}`. They are fine inside that app, and your files are stuck in it.

What I wanted was smaller and stricter: components, without giving up the plain file.

## What CompMD keeps

The one rule CompMD never breaks: **a CompMD file reads well in any Markdown renderer.** The components are written as
lines Markdown already shows as text, and their content is ordinary Markdown inside them. Open a `.comp.md` file on
GitHub and you read it in order, top to bottom. You lose the layout, never the words.

Here is a small one. First its source, then what it draws:

```compmd {render=both}
::: grid {min=11rem gap=m}
::: box {padding=m border=subtle border-radius=m}
**Tuesday.** The quay, before the boats come in.
::: /box

::: box {padding=m border=subtle border-radius=m}
**Thursday.** The orchard, if it doesn't rain.
::: /box
::: /grid
```

In a plain renderer, those `:::` lines show as they are, and the two days read one after the other. That's the whole
trade: a little noise in the plain view, a real layout everywhere else.

## What CompMD gives a writer

Once that rule holds, the rest can be generous.

- **Layout** with a handful of built-ins: `box`, `row`, `stack` and `grid`, sized and spaced with words like `s`, `m`
  and `l` rather than pixels, so a page looks right in a light theme and a dark one.
- **Data** that lives beside the document. A list in the frontmatter, a CSV, a YAML file: `each` lays it out, and
  `{{…}}` puts its values in the text. Update the spreadsheet, and the page follows.
- **Inputs** the reader changes, a filter or a choice, with the text following along. No script of your own.
- **Your own components, written in Markdown.** A card, a badge, a callout: you define it once, in a code block in
  the document or in a file of its own, and use it by name. Most components never need a line of code.
- **Packs** for the rest. A chart, a Mermaid diagram, a wireframe: a pack is a set of components on npm, which a
  document imports by name and version.

That last one is where the "trusting someone's code" problem comes back, so it got the most care. A component with
code runs in a sandbox, in a frame of its own, apart from the page and your data, unless the host that shows the
document says it can be trusted. By default it can't reach the network either. Opening a document never runs code that
can read your data. This isn't a feature added later: it's in
[the spec](https://compmd.dev/spec/), next to the syntax.

Here is data and a loop together, the reading list of a book club, kept in the frontmatter:

```compmd {render=both}
---
books:
  - { title: Klara and the Sun, by: Kazuo Ishiguro }
  - { title: The Overstory, by: Richard Powers }
  - { title: Piranesi, by: Susanna Clarke }
---

This season we read :value[three]{of=(string(size(md.doc.books)))} books:

::: each {in=(md.doc.books) as=book join=list}
- *{{book.title}}*, by {{book.by}}
::: /each
```

## Why a spec, and not just a tool?

Because I want my files to outlive any tool I use today. Markdown survived every editor that came and went because it
is a format, not a product. CompMD is a format too: a [written spec](https://compmd.dev/spec/) with test cases any
implementation can run, a processor in Rust that runs in the browser as WebAssembly, and nothing in it that assumes one
app. A blog, an editor, a notes app or a documentation site can all show the same file the same way.

It also means that when something is unclear, the answer goes in the spec, not in a corner of one app's code.

## Where it is today

CompMD is early. The spec is a draft and still changes, sometimes in ways that break documents written for an earlier
draft. But a lot already works, and you can try all of it without installing anything:

- [The playground](https://compmd.dev/playground/): write a document and see it drawn as you type, with many
  examples to start from.
- [The Author's Guide](https://compmd.dev/guides/authors/), for writing documents, and
  [the Developer's Guide](https://compmd.dev/guides/developers/), for building with CompMD, all the way to publishing
  a pack.
- [The code on GitHub](https://github.com/comp-markdown/compmd), where everything happens in the open.

## Conclusion

Markdown got one thing exactly right: the file is for people first. CompMD tries to keep that, and add just enough on
top for the pages Markdown can't make on its own: layout, data, components, safely.

If you write in Markdown and keep running into its edges, give the playground ten minutes. And if something feels wrong
or missing, tell me, in the comments below or in [an issue on GitHub](https://github.com/comp-markdown/compmd/issues).
This early on, a single good question can change where CompMD goes.

Until next time...

Hakan Serce
