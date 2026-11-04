# CompMD Author's Guide

How to write in [CompMD](https://compmd.hakanserce.com/): documents that read as Markdown and render as pages, with boxes, grids, values from data, inputs the reader changes, and components of your own, written in CompMD itself. It is for writers, not programmers: no code is needed. Building CompMD into an application is [the Developer's Guide](../developers/README.md).

The playground's _Learn CompMD_ tour is the five-minute introduction, hands on; this book is what you keep open while writing. Every example in it opens in the playground, where you can change it and watch the page.

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

**Part II. Data and inputs**

6. [Data from files](06-data-from-files.md)
7. [Inputs the reader changes](07-inputs-the-reader-changes.md)
8. [Dates, numbers and lists](08-dates-numbers-and-lists.md)

**Part III. Layout**

9. [Boxes, rows, stacks and grids](09-boxes-rows-stacks-and-grids.md)
10. [Widths and breakpoints](10-widths-and-breakpoints.md)
11. [Placing blocks](11-placing-blocks.md)

**Part IV. Components of your own**

12. [Definitions](12-definitions.md)
13. [Slots](13-slots.md)
14. [Template slots](14-template-slots.md)
15. [Signatures and fallbacks](15-signatures-and-fallbacks.md)

**Part V. Packs**

16. [Using a pack](16-using-a-pack.md)
17. [Code blocks a pack draws](17-code-blocks-a-pack-draws.md)
18. [Components in code, in brief](18-components-in-code-in-brief.md)

**Appendices**

19. [The syntax at a glance](19-the-syntax-at-a-glance.md)
20. [The diagnostics and their fixes](20-the-diagnostics-and-their-fixes.md)

## The examples

Each chapter's document is a file in [`examples/`](examples), which the guides' tests process and compare with the chapter's text. Every CompMD block in the book processes without a diagnostic, so what you copy works.
