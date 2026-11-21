# @comp-markdown/figure

A [CompMD](https://compmd.dev/) pack with one component, `figure`: an image with its caption under it, for a post's photo, a newsletter's chart or a recipe's dish. The caption is written once, and is the image's text alternative too. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 5 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

:: figure[Boats at the quay, the lamps just lit.] {src="images/harbour.jpg"}
```

- **The caption** is the text in `[…]`. The image's alt is the same text, so a screen reader and a reader without images get the caption.
- **The image is your document's:** `images/harbour.jpg` is next to the document that uses the figure, wherever the figure comes from.
- **Its box** has a little space above and below. A use changes it as any box's: `{width=20rem}`, `padding`, `border`.
- **On paper** it prints as it shows, the image and its caption.
- **In a plain Markdown reader,** and on GitHub, a figure is its caption.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/figure dev     # the test page: demo.comp.md, two captioned images
pnpm --dir compmd-packs/figure test    # the publish check, and the figure's expanded tree
pnpm --dir compmd-packs/figure lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
