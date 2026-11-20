---
md:
  export:
    pack: "@comp-markdown/card"
    version: "0.1.0"
    names: [card]
---

# @comp-markdown/card

`card` is a box with an image on top, then what you write in it, then a footer: a post's teaser, a recipe, a place to visit. Cards in a `grid` make a row of them that wraps.

```md
::: card
### The harbour at dusk

A walk along the old quay, an hour before the boats come in.

:: #image
![Boats at the quay, the lamps just lit](harbour.jpg)

:: #footer
[Read the walk](harbour.comp.md)
::: /card
```

The image is a slot, not a param: it is your document's own image, written as Markdown, so its path is your document's and its alt text says what it shows. The title is the heading you write in the card, at the level your document needs. A card with no image or no footer leaves it out.

Its box has a subtle border, rounded corners and some padding; a use changes them as any box's: `::: card {background=subtle border=none}`. In a plain Markdown reader, a card is its text, then its image, then its footer.

```compmd component=card {padding=m border=subtle border-radius=m}
slots: [default, image, footer]
---
{{md.slot.image}}

{{md.slot}}

{{md.slot.footer}}
```
