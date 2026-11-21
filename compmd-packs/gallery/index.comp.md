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

`gallery` is a grid of cards, one for each row of a list: a post's photos, a trip's stops, a shop's products. Each card has the row's image on top, its title and its description.

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

with `walks.yaml`:

```yaml
- title: The harbour at dusk
  description: A walk along the old quay, an hour before the boats come in.
  image: images/harbour.jpg
  alt: Boats at the quay, the lamps just lit
```

Each row has a `title`, an `image` with its `alt` text, and a `description`, which a row may leave out. The images are your document's: `images/harbour.jpg` is next to the document that uses the gallery, wherever the gallery comes from. The cards are at least 14rem wide, as many in a row as the column holds, and wrap. The cards' titles are headings one level below the heading above the gallery.

In a plain Markdown reader, a gallery is its sentence, `Three walks around the harbour`.

```compmd component=gallery
params:
  rows: array
---
::: grid {min=14rem gap=m}
::: each {in=(rows) as=row}
::: card
# {{row.title}}

::: if {when=(has(row.description))}
{{row.description}}
::: /if

:: #image
![{{row.alt}}]({{row.image}})
::: /card
::: /each
::: /grid
```
