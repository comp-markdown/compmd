---
title: "Hello, world: introducing CompMD"
date: 2026-11-19
updated: 2026-11-20
summary: Markdown with components, which still reads as Markdown everywhere else. A first look, and what this blog will hold.
tags: [about]
---

[CompMD](https://compmd.dev) is Markdown with components. You write a post, a newsletter or your notes as you would in any Markdown, and
reach for a component where plain Markdown runs out: a layout, a list made from data, a chart, a diagram. Open the same
file on GitHub or in any Markdown editor, and it still reads well.

CompMD is still in an early stage, but development is progressing fast on [GitHub](https://github.com/comp-markdown/compmd). It has a detailed [spec](https://compmd.dev/spec/), with its security model part of it from the start, has a [playground](https://compmd.dev/playground/) with many examples to try out what it can do already, and guides for [authors](https://compmd.dev/guides/authors/) and [developers](https://compmd.dev/guides/developers/). We even have a lightweight static blog generator for CompMD posts, [@compmd/blog](https://www.npmjs.com/package/@compmd/blog). A lot more is coming up very soon.

In this post we will have a very quick look at some examples, and there are many more in the links above. Feel free to roam around the website.

## A first look

Let's say you are preparing a book club's page, and would like to have it a bit more colorful and interactive, and also want to keep it maintainable. You get the looks with a small shelf component, and you separate the books data to a CSV that you maintain using a spreadsheet app separately. Check this out below, first the `.comp.md` source, and then how it looks. _Check [the playground](https://compmd.dev/playground/?doc=book-club) for a fuller version of this example._

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

A layout is a few lines too. Three steps side by side on a wide screen, one under another on a phone:

```compmd {render=both}
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
```

You can avoid repetition with `each`, which loops through a list. See below an example where we put the two together: the posts this blog has coming, written once in the frontmatter and laid out as cards.

```compmd {render=both}
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
```

In a plain Markdown renderer, the `:::` lines show as text and the rest reads in order: nothing is lost, only the
layout. That is the promise CompMD keeps, whatever a document uses.

## What's next

CompMD is early stage, and a lot of development is focusing on fixing gaps and adding development tools, editor integrations and finalizing the v1 spec. I would really appreciate any constructive feedback from Markdown authors, note taking enthusiasts, Markdown integration developers, in the comments below or in [an issue on GitHub](https://github.com/comp-markdown/compmd/issues). Your feedback here can help us prioritize what to focus on in the early days and you can make a big impact in the direction we take.

Until next time...

Hakan Serce
