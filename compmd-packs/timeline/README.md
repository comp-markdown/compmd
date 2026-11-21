# @comp-markdown/timeline

A [CompMD](https://compmd.dev/) pack with two components, `timeline` and its `event`s: dates and what happened, along a line, for a harbour's history in a post, a project's milestones in notes, a trip day by day. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 8 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

::: timeline
::: event {date=1897 title="The harbour wall"}
Built in a single summer, of granite from the quarry.
::: /event

::: event {date=2024 title="The lamps come back" tone=accent}
All forty of them, restored.
::: /event
::: /timeline
```

- **Each `event`** has a dot on the line, its `date` small and muted, its `title` in bold (an event may leave it out), then what you write in it: paragraphs, links, a list. The date is written as you write it: `1897`, `May 3`, `Day 2`.
- **`tone=accent`** fills an event's dot with the accent colour, for the current or most important one.
- **From data,** an `each` inside the timeline writes the events: `::: event {date=(m.date) title=(m.title)}` around `{{m.what}}`.
- **A quick timeline** is a list with no events, `- **1897**: the harbour wall is built.`, the list's markers on the line.
- **The line** is the timeline's start border: `{border-inline-start=subtle}` for a fainter one. Its room, `padding-inline-start`, is what places the dots, so leave it as it is.
- **On paper** it prints as it shows.
- **In a plain Markdown reader,** and on GitHub, a timeline is its events' text, each date, title and text in order.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/timeline dev     # the test page: demo.comp.md, each use as an example
pnpm --dir compmd-packs/timeline test    # the publish check, and the timeline's expanded tree
pnpm --dir compmd-packs/timeline lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
