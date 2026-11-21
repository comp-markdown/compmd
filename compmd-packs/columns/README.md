# @comp-markdown/columns

A [CompMD](https://compmd.dev/) pack with one component, `columns`: two or three columns side by side, for a recipe's ingredients beside its method, a newsletter's two stories, a story between two notes. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 4b of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-end}
## Ingredients

- 4 apples
- 100 g butter

:: #end
## Method

Peel and slice the apples, then...
::: /columns
```

- **The columns:** what you write first is the first column; `:: #middle` starts a middle one and `:: #end` the last. A column holds any blocks, with no `box` around them.
- **`split`** sets the widths: `even` (the default), `wide-start` or `wide-end` (that column twice the others), or `wide-middle` with three.
- **Side by side from `sm` up,** 480 px of their box, and stacked in order below it, on a phone.
- **`gap`** is the space between them, `m` unless a use says otherwise: `{gap=l}`.
- **Start and end** follow the text's direction: in an Arabic document (`md.lang: ar`) the first column is on the right, as it is for `{lang=ar dir=rtl}` on columns in Arabic in another language's page.
- **Their box** is a grid. A use changes it as any box's: `{padding=m border=subtle}`.
- **In a plain Markdown reader,** and on GitHub, the columns are their text, one after the other.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/columns dev     # the test page: demo.comp.md, each use as an example
pnpm --dir compmd-packs/columns test    # the publish check, the expanded tree, and the page in Chrome and WebKit
pnpm --dir compmd-packs/columns lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
