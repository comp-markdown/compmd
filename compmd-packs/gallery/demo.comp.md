---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
---

Each example of `gallery` shows its source, then what it draws. Their lists are `walks.yaml`, `photos.yaml` and `shapes.yaml`, beside this page.

# From a list in data

Each card at least 14rem wide, as many side by side as fit.

````compmd {render=both}
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
````

# Written in the document

With an item that has no description.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
---

:: gallery[Two walks] {items=([{"title": "The old orchard", "description": "Apples in October.", "image": "images/orchard.svg", "alt": "Rows of apple trees on a slope"}, {"title": "Along the canal", "image": "images/canal.svg", "alt": "A canal under a low stone bridge"}])}
````

# A dozen photos

`min=10rem`, so more fit side by side, with a small `gap`.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./photos.yaml
      as: photos
---

:: gallery[Twelve photos from the harbour] {items=(md.data.photos) min=10rem gap=s}
````

# Exactly two side by side

`columns=2`, however wide the page.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./walks.yaml
      as: walks
---

:: gallery[Three walks, two side by side] {items=(md.data.walks) columns=2}
````

# One under another on a phone, three side by side from `md` up

`columns=1 md:columns=3`, with more space between them.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./walks.yaml
      as: walks
---

:: gallery[Three walks, stacked on a phone] {items=(md.data.walks) columns=1 md:columns=3 gap=l}
````

# Photos of different shapes, cropped to one

`ratio=4/3`: a tall photo and a wide one take the same shape as the others, covering it.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./shapes.yaml
      as: shapes
---

:: gallery[Three photos of different shapes, each cropped to 4:3] {items=(md.data.shapes) ratio=4/3}
````

# The whole of each photo

`image-fit=contain`: each photo is as large as fits in the shape, none of it cut, and `image-position="start center"` puts it at the shape's top, in its middle.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./shapes.yaml
      as: shapes
---

:: gallery[Three photos of different shapes, each whole in a 4:3 shape] {items=(md.data.shapes) ratio=4/3 image-fit=contain image-position="start center"}
````
