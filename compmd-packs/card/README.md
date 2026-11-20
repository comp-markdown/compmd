# @comp-markdown/card

A [CompMD](https://compmd.dev/) pack with one component, `card`: a box with an image on top, then what you write in it, then a footer, for a post's teaser, a recipe or a place to visit. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is the first of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md) (item 4).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: grid {min=14rem gap=m}
::: card
### The harbour at dusk

A walk along the old quay, an hour before the boats come in.

:: #image
![Boats at the quay, the lamps just lit](harbour.jpg)

:: #footer
[Read the walk](harbour.comp.md)
::: /card
::: /grid
```

- **The image is a slot,** `:: #image`, not a param: it is your document's own image, written as Markdown, so its path is relative to your document and its alt text says what it shows. A card without an image leaves it out.
- **The title is your heading,** at the level your document needs; a card adds none.
- **The footer,** `:: #footer`, is for a link or a date; a card without one leaves it out.
- **Its box** has a subtle border, rounded corners and some padding. A use changes them as any box's: `::: card {background=subtle border=none}`, `width`, `padding`.
- **In a plain Markdown reader,** and on GitHub, a card is its text, then its image, then its footer.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/card dev     # the test page: demo.comp.md, three cards in a grid, with a light and dark switch
pnpm --dir compmd-packs/card test    # the publish check, and the card's expanded tree and plain reading
pnpm --dir compmd-packs/card lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
