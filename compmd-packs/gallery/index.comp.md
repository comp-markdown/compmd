---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
  export:
    pack: "@comp-markdown/gallery"
    version: "0.1.0"
    names: [gallery]
---

# @comp-markdown/gallery

`gallery` is a grid of cards, one for each item of a list: a post's photos, a trip's stops, a shop's products. Each card has the item's image on top, its title and its description.

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

with `walks.yaml`:

```yaml
- title: The harbour at dusk
  description: A walk along the old quay, an hour before the boats come in.
  image: images/harbour.jpg
  alt: Boats at the quay, the lamps just lit
```

Each item has a `title`, an `image` with its `alt` text, and a `description`, which an item may leave out. The images are your document's: `images/harbour.jpg` is next to the document that uses the gallery, wherever the gallery comes from. The cards are at least `min` wide (14rem), as many side by side as the column holds, and wrap; `columns=3` puts exactly three side by side instead, and takes breakpoints, `columns=1 md:columns=3`. `gap` is the space between them (`m`). The cards' titles are headings one level below the heading above the gallery.

In a plain Markdown reader, a gallery is its sentence, `Three walks around the harbour`.

```compmd component=gallery
params:
  items: array
  min:
    type: length
    default: 14rem
    doc: The narrowest a card gets; as many side by side as the column holds
  columns:
    type: number?
    doc: Exactly this many cards side by side, in place of min
  gap:
    type: space
    default: m
    doc: The space between the cards
---
::: grid {min=(columns == null ? min : null) grid-template-columns=(columns == null ? null : "repeat(" + string(int(columns)) + ", minmax(0, 1fr))") gap=(gap)}
::: each {in=(items) as=item}
::: card
# {{item.title}}

::: if {when=(has(item.description))}
{{item.description}}
::: /if

:: #image
![{{item.alt}}]({{item.image}})
::: /card
::: /each
::: /grid
```
