+++
id = "01a1fe7c-a607-714d-a899-b2e4baa1f33a"
number = 45
title = "Component: gallery"
status = "approved"
tasks = ["CMD-315"]
author = "claude:79612601"
reviewers = []
created = 2026-11-21T14:22:24Z
updated = 2026-11-21T15:05:14Z
+++

`@comp-markdown/gallery`, item 4 of [the first components](docs/first-components.md), as [decision 7 of the no-code components](docs/no-code-components.md) has it: a leaf over data rows, each a `card` made in the definition with `each`.

## The definition

```compmd component=gallery
params:
  rows: array
---
::: grid {min=14rem gap=m}
::: each {in=(rows) as=row}
::: card
# {{row.title}}

::: if {when=(has(row.description))}
{{row.description}}
::: /if

:: #image
![{{row.alt}}]({{row.image}})
::: /card
::: /each
::: /grid
```

- **The cards are `@comp-markdown/card`'s,** imported by the gallery's document and named in its `dependencies`. The image goes in the card's `image` slot, since the card takes its image there, not inline as the design's sketch had it.
- **The images are the caller's:** `{{row.image}}` starts with an `each` name over a param, so CMD-362 resolves it against the document that uses the gallery. A test puts that document in a folder (`posts/`) and checks it.
- **A row may leave out its description** (`has(row.description)`). The title, the image and its alt are each row's.
- **The titles are `#`,** relative to the use: one level below the heading above the gallery.
- **The fallback is the use's sentence,** which a plain reader shows.

## Two choices to check

1. **The grid is `::: grid {min=14rem gap=m}` in the body**, as the design wrote it, so a use's params style the gallery's box around the grid, not the grid itself.
2. **The image is required in every row.** The card's image is a slot, and a slot can't be placed under an `if`, so a row without an image would get an empty image. Leaving it out would need the card to take its image another way.

## The rest

A demo over `walks.yaml`, the card demo's two pictures and a third; tests of the expanded tree (the grid, each card's image, heading and text, a row without a description, the caller's paths), the plain reading, and the demo. The pack is in the workspace and `compmd-packs/versions.json` at 0.1.0.
