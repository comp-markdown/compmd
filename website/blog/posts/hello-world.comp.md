---
title: "Hello, world: introducing CompMD"
date: 2026-11-19
summary: Markdown with components, which still reads as Markdown everywhere else. A first look, and what this blog will hold.
tags: [about]
---

CompMD is Markdown with components. You write a post, a newsletter or your notes as you would in any Markdown, and
reach for a component where plain Markdown runs out: a layout, a list made from data, a chart, a diagram. Open the same
file on GitHub or in any Markdown editor, and it still reads well.

## A first look

A book club's page, as its organizer might write it: a paragraph, a shelf of the books the club has read, and what
it made of the one picked. Each example here shows the document, then what it makes. Pick a cover.

````compmd {render=both}
---
md:
  import:
    - from: ./hello-world/shelf.js
      names: [shelf]
  data: [{from: ./hello-world/club.csv, as: books}]
  inputs:
    book: string = "Klara and the Sun"
---

# Thursday Book Club

We meet on the last Thursday of the month, at Ana's.
Pick a cover to see what we made of it.

:: shelf[] {books=(md.data.books) picked=book}

::: each {in=(md.data.books.filter(b, b.title == md.input.book)) as=b}
**{{b.title}}**, {{b.author}}. {{b.verdict}}

::: if {when=(b.status == "read")}
The club gave it :rating[4 of 5]{value=(b.rating)}

:: #else
Not rated yet.
::: /if
::: /each

```compmd component=shelf signature
params:
  books: array
  picked: input<string>
```
````

The books are a spreadsheet beside the page, the shelf is a small script, and the rest is the text a club would write
anyway. In a plain Markdown renderer, it all still reads in order.

A layout is a few lines too. Three steps side by side on a wide screen, one under another on a phone:

````compmd {render=both}
::: grid {min=7rem gap=s}
::: box {padding=s background=subtle border-radius=m}
**Plan**
::: /box
::: box {padding=s background=subtle border-radius=m}
**Build**
::: /box
::: box {padding=s background=accent border-radius=m}
**Ship**
::: /box
::: /grid
````

And the two together: the posts this blog has coming, written once in the frontmatter and laid out as cards.

````compmd {render=both}
---
coming:
  - title: Why CompMD
    about: what it gives a writer that plain Markdown doesn't, and what it keeps
  - title: Where CompMD is
    about: what works today, and what comes next
  - title: MDX and CompMD
    about: two answers to components in Markdown, side by side
---

::: grid {min=12rem gap=m}

::: each {in=(md.doc.coming) as=post}

::: box {padding=m border=subtle border-radius=m}

**{{post.title}}**: {{post.about}}.

::: /box

::: /each

::: /grid
````

In a plain Markdown renderer, the `:::` lines show as text and the rest reads in order: nothing is lost, only the
layout. That is the promise CompMD keeps, whatever a document uses.

## What comes here

The spec says what CompMD is, and the guides how to write it and build with it. This blog is for the rest: why CompMD
is as it is, where it is going, how it compares with what else there is, and what each release brings. Every post is a
CompMD document like the one above, in [the repository](https://github.com/comp-markdown/compmd/tree/main/website/blog).

To try CompMD now, open [the playground](/playground/), or start with [the Author's Guide](/guides/authors/).
