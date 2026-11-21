# An image cropped to its box

**Status: proposed (2026-11-21, CMD-366).** How a document, or a `compmd` definition, gives an image a shape of its own:
a gallery whose photos are all 4:3, a card whose image is a banner, a figure that crops a tall photo. Found in CR-45:
the gallery's cards are as uneven as their images, and CompMD has no way to make them one shape. CMD-367 uses it in
the gallery.

**Terms.** An **image box** is a box (`box`, or a component's box) whose content, once expanded, is one image and
nothing else: one paragraph that holds one Markdown image, alone or as a link's only content, white space aside. Its
**inside** is the box less its border and padding (as in [sizing a component](sizing-components.md)).

## Decisions

1. **An image box fits its image as a component's box fits a drawing** (spec §7.4): the image takes the box's inside,
   and `object-fit` and `object-position` on the box say how it sits in it, `contain` and `center` by default, with
   CSS's meanings. So `::: box {aspect-ratio=4/3 object-fit=cover}` around `![…](harbour.jpg)` crops the photo to 4:3,
   whatever its own shape. Today they mean nothing to a block that isn't a host-language component (§9.2), so no
   document changes meaning.
   - **The box's height** is what crops: `aspect-ratio`, `height`, `fill`, or a layout that stretches it (a grid row).
     Without one, the box is as tall as the image at the box's width, as now, and nothing is cropped.
   - **Rejected: a param list on the image**, `![…](harbour.jpg){aspect-ratio=4/3}`. It is syntax Markdown doesn't
     have, which a plain reader shows as text after the image.
   - **Rejected: an `image` component**, `:: image[Boats at dusk] {src=… aspect-ratio=4/3}`. A plain reader shows its
     alt text, not the image, and an author writes images two ways.
2. **It is found on the expanded tree,** so a definition can make one around content it is given:
   `::: box {aspect-ratio=(ratio) object-fit=cover}` around `{{md.slot.image}}` in the card, or around
   `![{{item.alt}}]({{item.image}})` in the gallery. The renderer marks such a box, and its CSS sizes the image.
3. **`object-fit` and `object-position` on a box that is neither** a host-language component's nor an image box mean
   nothing still, and are now **W125**, at the param, so an author sees why nothing changed (a box with an image and
   a caption, say: the image needs a box of its own).
4. **No breakpoint, as now** (§9.2): they stay written without one. The box's `aspect-ratio` takes breakpoints, so a
   gallery can crop to 1:1 on a phone and 4:3 from `md` up.
5. **Print, and plain readers:** the image prints as it shows, cropped. A plain Markdown reader shows the image whole,
   since it ignores the box.

## In use

````md
::: grid {min=12rem gap=s}
::: box {aspect-ratio=1 object-fit=cover}
![Boats at the quay](harbour.jpg)
::: /box

::: box {aspect-ratio=1 object-fit=cover object-position="center start"}
![The orchard, a tall photo](orchard.jpg)
::: /box
::: /grid
````

Both photos are squares, the second one cropped from its top. A gallery (CMD-367) takes the ratio as a param, and its
cards' images are one shape.

## What changes

- **The spec:** §9.2 (`object-fit` and `object-position` also fit an image box's image; the image box defined), §10
  (W125), §12.3 (nothing new in the tree: the params are the box's, as now). The changelog. Conformance: an image box
  with each fit, a linked image, a box that isn't one (W125), and one made in a definition around a slot.
- **The processor:** W125, from the expanded tree.
- **The renderer:** a box whose children are one image is marked (`compmd-image-box`), and its CSS gives the image
  the box's inside, `object-fit` and `object-position`, in the page and in print. Browser tests in Chrome and WebKit:
  a photo cropped to 1:1 and 16:9 by `cover`, letterboxed by `contain`, placed by `object-position`, printed.
- **The guides:** the Author's Guide's chapter on images and sizes.

## Gaps

- **G1. Which image of several:** a box with two images, or an image and text, is no image box. A figure's image and
  caption need a box around the image alone, which its definition writes.
- **G2. Raw HTML `<img>`** isn't a Markdown image, so a box around one isn't an image box. Kept out: raw HTML keeps
  its own attributes.
