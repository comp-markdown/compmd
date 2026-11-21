# @comp-markdown/timeline

A [CompMD](https://compmd.dev/) pack with one component, `timeline`: dates and what happened, along a line, for a harbour's history in a post, a project's milestones in notes, a trip day by day. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 8 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline]
---

::: timeline
- **1897**: the harbour wall is built.
- **1952**: the quay is rebuilt after the storm.
- **2024**: the lamps along the quay come back.
::: /timeline
```

- **The timeline** is the list you write in it, each item a date and what happened, the date as you write it: `**1897**`, `**May 3**`, `**Day 2**`. An item holds what a list item holds: paragraphs, links, a nested list.
- **From data,** an `each` inside it writes the items, `join=list` making them one list: `::: each {in=(md.data.milestones) as=m join=list}` around `- **{{m.date}}**: {{m.what}}`.
- **Its box** has a line down its start side and room beside it. A use changes it as any box's: `{border-inline-start=subtle}` for a fainter line, `{padding-inline-start=l}` for more room.
- **On paper** it prints as it shows.
- **In a plain Markdown reader,** and on GitHub, a timeline is its list.

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
