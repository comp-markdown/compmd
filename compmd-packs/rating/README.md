# @comp-markdown/rating

A [CompMD](https://compmd.dev/) pack with one component, `rating`: a value shown as stars, half stars included, for a review's score, a recipe's difficulty or a book's place on a reading list. It shows a value; it isn't an input. The stars are drawn as SVG in the host's strict sandbox. It is item 9 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

:: rating[Four of five stars] {value=4}

The cheese was :rating[three and a half of five]{value=3.5 width=5rem} at best.
```

- **`value`** is shown to the nearest half star, between 0 and `of`.
- **`of`** is how many stars there are, 5 unless a use says otherwise, 20 at most.
- **The text in `[…]`** is what a reader without the component sees, and the stars' text alternative. Without one, they say "Rated 4 of 5".
- **Its box** is 7.5rem wide unless a use says otherwise (`width`, `height`); the stars fit it.
- **On paper** the stars print as drawn, and they follow the page's colours.
- **In a plain Markdown reader,** and on GitHub, a rating is its text.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/rating dev     # the test page: demo.comp.md, each param as an example
pnpm --dir compmd-packs/rating test    # the publish check, and the page in Chrome and WebKit
pnpm --dir compmd-packs/rating lint    # the type check
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
