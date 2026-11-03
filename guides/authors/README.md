# CompMD Author's Guide

How to write in [CompMD](https://compmd.hakanserce.com/): documents that read as Markdown and render as pages, with boxes, grids, values from data, inputs the reader changes, and components of your own, written in CompMD itself. It is for writers, not programmers: no code is needed. Building CompMD into an application is [the Developer's Guide](../developers/README.md).

The playground's *Learn CompMD* tour is the five-minute introduction, hands on; this book is what you keep open while writing. Every example in it opens in the playground, where you can change it and watch the page. On the website it is at [compmd.hakanserce.com/guides/authors/](https://compmd.hakanserce.com/guides/authors/).

## Who it is for

People who write: notes, newsletters, reports, documentation, anything you would write in Markdown and want to do more with. You don't need to program. The book uses a few small formulas, to compute a total or choose a color, and explains each one where it first appears. It covers the syntax, data, inputs, layout, components written in CompMD itself, and the packs a document uses.

Components written in code, in React, Svelte or JavaScript, are [the Developer's Guide](../developers/README.md)'s subject. So are applications that show CompMD documents. This book says only what such a component looks like to the document that uses it.

## Contents

**Part I. Writing**

1. [A first document](01-a-first-document.md)
2. [The Markdown you know](02-the-markdown-you-know.md)
3. [Containers and params](03-containers-and-params.md)
4. [Values and expressions](04-values-and-expressions.md)
5. [Showing and repeating](05-showing-and-repeating.md)

**Part II. Data and inputs** *(to come)*

6. Data from files
7. Inputs the reader changes
8. Dates, numbers and lists

**Part III. Layout** *(to come)*

9. Boxes, rows, stacks and grids
10. Widths and breakpoints
11. Placing blocks

**Part IV. Components of your own** *(to come)*

12. Definitions
13. Slots
14. Template slots
15. Signatures and fallbacks

**Part V. Packs**

16. [Using a pack](16-using-a-pack.md)
17. Code blocks a pack draws
18. Components in code, in brief

**Appendices** *(to come)*: the syntax at a glance, the diagnostics and their fixes.

## The examples

Each chapter's document is a file in [`examples/`](examples), which the guides' tests process and compare with the chapter's text. Every CompMD block in the book processes without a diagnostic, so what you copy works.

## Reading it on your machine

The chapters read as they are on GitHub, and their **Try it** links open the playground on the website. To see the book as the website shows it, with the playground on your machine, run these from the repository's root, after `pnpm install`:

- **Live, as you edit:** `pnpm --dir editor build` once, for the playground, then `pnpm --dir website dev`. Open [localhost:5173/guides/authors/](http://localhost:5173/guides/authors/); a page reloads when its chapter changes.
- **As it will be published:** `pnpm --dir editor build`, `pnpm --dir website build`, then `pnpm --dir website preview`, and open the address it prints.
- **The tests:** `pnpm --dir guides test` checks every chapter against its examples.
