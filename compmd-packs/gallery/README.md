# @comp-markdown/gallery

A [CompMD](https://compmd.dev/) pack with one component, `gallery`: a grid of cards, one for each item of a list, each with its image, its title and its description, for a post's photos, a trip's stops or a shop's products. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. Its cards are [`@comp-markdown/card`](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/card/README.md)'s. It is item 4 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

## Use it in a document

```md
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./walks.yaml
      as: walks
---

:: gallery[Three walks around the harbour] {items=(md.data.walks)}
```

with `walks.yaml` beside the document:

```yaml
- title: The harbour at dusk
  description: A walk along the old quay, an hour before the boats come in.
  image: images/harbour.jpg
  alt: Boats at the quay, the lamps just lit
- title: Along the canal
  image: images/canal.jpg
  alt: A canal under a low stone bridge
```

- **Each item** has a `title`, an `image` with its `alt` text, and a `description`, which an item may leave out. The items can be written in the document too: `items=([{"title": "…", "image": "…", "alt": "…"}])`.
- **The images are your document's:** `images/harbour.jpg` is next to the document that uses the gallery, wherever the gallery comes from.
- **The cards** are at least `min` wide (14rem), as many side by side as the column holds, and wrap: `min=10rem` packs a dozen photos. `columns=2` puts exactly two side by side instead, and takes breakpoints like any param: `columns=1 md:columns=3` stacks them on a phone and puts three side by side from `md` up. `gap` is the space between them (`m`). Every card is as tall as the tallest, across rows too. Their titles are headings one level below the heading above the gallery.
- **One shape for every image:** `ratio=4/3` crops each image to 4:3, so photos of different sizes make an even grid. `image-fit` says how an image fills the shape, `cover` (the default) or `contain` to show all of it, and `image-position` where it sits, one or two of `start`, `center` and `end`, top to bottom first (`"start center"` for its top). Without `ratio`, each image keeps its own shape.
- **In a plain Markdown reader,** and on GitHub, a gallery is its sentence: `Three walks around the harbour`.

A zoom or a lightbox on a card's image waits for components that read the document's files.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/gallery dev     # the test page: demo.comp.md, an example of each param, its source shown
pnpm --dir compmd-packs/gallery test    # the publish check, and the gallery's expanded tree
pnpm --dir compmd-packs/gallery lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.2.0:** `ratio`, `image-fit` and `image-position` crop every image to one shape; every card is as tall as the tallest, across rows too.
- **0.1.0:** the first version.
