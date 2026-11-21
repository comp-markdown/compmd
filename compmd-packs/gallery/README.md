# @comp-markdown/gallery

A [CompMD](https://compmd.dev/) pack with one component, `gallery`: a grid of cards, one for each row of a list, each with its image, its title and its description, for a post's photos, a trip's stops or a shop's products. It is a `compmd` definition, so it has no code: every CompMD processor renders it, with no sandbox and nothing to load. Its cards are [`@comp-markdown/card`](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/card/README.md)'s. It is item 4 of [the first components](https://github.com/comp-markdown/compmd/blob/main/docs/first-components.md).

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

:: gallery[Three walks around the harbour] {rows=(md.data.walks)}
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

- **Each row** has a `title`, an `image` with its `alt` text, and a `description`, which a row may leave out. The rows can be written in the document too: `rows=([{"title": "…", "image": "…", "alt": "…"}])`.
- **The images are your document's:** `images/harbour.jpg` is next to the document that uses the gallery, wherever the gallery comes from.
- **The cards** are at least 14rem wide, as many in a row as the column holds, and wrap. Their titles are headings one level below the heading above the gallery.
- **In a plain Markdown reader,** and on GitHub, a gallery is its sentence: `Three walks around the harbour`.

A zoom or a lightbox on a card's image waits for components that read the document's files.

## Try it

From the repository (`pnpm install` at its root first):

```sh
pnpm --dir compmd-packs/gallery dev     # the test page: demo.comp.md, three walks from walks.yaml
pnpm --dir compmd-packs/gallery test    # the publish check, and the gallery's expanded tree
pnpm --dir compmd-packs/gallery lint    # the type check of the tests
```

## Licence

MIT.

## Releases

The pack has its own version, in `compmd-packs/versions.json`, released with `cargo make release-packs` (`RELEASING.md`).

- **0.1.0:** the first version.
