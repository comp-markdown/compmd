# @comp-markdown/link-card

A [CompMD](https://compmd.dev/) pack with one component, `link-card`: a card that is a link, its title and a line under it, and a click anywhere on it follows the link. For a post's "read next", a newsletter's stories, a trip's walks. It is a `compmd` definition on [`card`](../card/), so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. It is item 4 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/link-card@0"
      names: [link-card]
---

:: link-card[The harbour at dusk] {href="harbour.comp.md" description="A walk along the old quay, an hour before the boats come in."}
```

- **The title** is the text in `[…]`, and the link's text: a screen reader names the link by it. It is a heading one level below the heading above the card.
- **`href` is your document's:** `harbour.comp.md` is next to the document that uses the card, wherever the card comes from.
- **`description`** is the line under the title; a use may leave it out.
- **The whole card** takes the click, as its box has `stretch-link`.
- **Its box** is a card's, a subtle border, rounded corners and some padding. A use changes it as any box's: `{width=20rem}`, `background`. Link cards in a `grid` make a row that wraps.
- **In a plain Markdown reader,** and on GitHub, a link card is its line as written, its title in it.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/link-card dev     # the test page: demo.comp.md, each use as an example
pnpm --dir compmd-packs/link-card test    # the publish check, and the link card's expanded tree
pnpm --dir compmd-packs/link-card lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
