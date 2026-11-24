---
title: "The first packs: cards, galleries, notes and more"
date: 2026-11-23
summary: Nine components for the pages people actually write (a card, a gallery, a sticky note, a timeline and more), eight of them written in CompMD itself, with no code at all.
tags: [packs, components]
---

CompMD is still very young. It promises Markdown with components, yet so far it has hardly any components ready to
use: a few built-ins, and the Mermaid and wireframe renderers. Don't get me wrong, CompMD has everything it takes to
build components, but an author expects the basics, cards, galleries, sticky notes, to be there already. So one of the
first things we are doing, over the next week or two, is making an initial set of components as CompMD packs, so that
authors can write documents with rich components right away.

After a quick look at similar component ecosystems, we picked this list to get CompMD's component library going:

| Component     | What it is                                                               |
| ------------- | ------------------------------------------------------------------------ |
| `tabs`        | Content behind labeled tabs; tabs of code blocks make a code group       |
| `details`     | A section that opens and closes under its summary                        |
| `steps`       | Numbered steps joined by a line, for a how-to                            |
| `card`        | A box with an image on top, your text, and a footer                      |
| `link-card`   | A card that is a link, clickable anywhere                                |
| `gallery`     | A grid of cards, one for each item of a list                             |
| `columns`     | Two or three columns side by side, stacked on a phone                    |
| `figure`      | An image with its caption, which is its alt text too                     |
| `badge`       | A word or two in a small tinted box, inside a sentence                   |
| `sticky-note` | A yellow note pinned on the page, over the text                          |
| `timeline`    | Dates and what happened, along a line                                    |
| `rating`      | Stars from a value, half stars included                                  |
| `chart`       | Bar, line, area, pie and other charts, from a document's data            |
| `word-cloud`  | Words sized by how often they come up                                    |
| `calendar`    | A month with its dates marked and their events listed                    |
| `weather`     | A forecast, from a weather service that needs no key                     |
| `graphviz`    | Diagrams written in Graphviz's `dot` language                            |
| `data-table`  | A table of data the reader can sort and search                           |
| `jq`          | A jq query over JSON, shown with its output                              |
| `music`       | Sheet music from ABC notation, which plays offline                       |
| `chord`       | A guitar or ukulele chord diagram, from its name                         |
| `chord-sheet` | Lyrics with their chords above them, from ChordPro, transposable         |
| `guitar-tab`  | Guitar tabs and scores, from Guitar Pro files or alphaTex                |
| `qr-code`     | A QR code for a link, for newsletters and posters printed on paper       |
| `embeds`      | YouTube, Spotify, Vimeo, SoundCloud, Bluesky, X and CodePen, in the page |

We are building this first set in a few iterations. In this one, with CompMD 0.10.0, we are releasing nine of them:
`card`, `link-card`, `gallery`, `figure`, `columns`, `badge`, `sticky-note`, `timeline` and `rating`.

The twist: eight of the nine have no code. They are written in CompMD itself, a few lines of Markdown with a signature
on top, so every CompMD processor renders them, with no sandbox and nothing to load. You could read one in a minute and
write your own in five.

## A card, and cards that are links

A `card` is a box with an image on top, then whatever you write, then a footer. The image is your own Markdown image,
so its path and its alt text are yours, and the title is your own heading, at the level your page needs.

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/card@0"
      names: [card]
---

::: card
### The harbour at dusk

A walk along the old quay, an hour before the boats come in.

:: #image
![Boats at the quay, the lamps just lit](first-packs/harbour.svg)

:: #footer
[Read the walk](#harbour)
::: /card
```

For a "read next" row, `link-card` is a card that is a link: click anywhere on it and you follow it, while a screen
reader still names the link by its title. Put a few in a `grid` and they wrap, each as tall as the tallest.

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/link-card@0"
      names: [link-card]
---

::: grid {min=12rem gap=m grid-auto-rows=1fr}
:: link-card[The harbour at dusk] {href="#harbour" description="An hour before the boats come in."}
:: link-card[The old orchard] {href="#orchard" description="Apples in October."}
:: link-card[Along the canal] {href="#canal" description="Twelve bridges, and a pub at the ninth."}
::: /grid
```

## A gallery, from a list

A `gallery` is a grid of cards, one for each item of a list: a post's photos, a trip's stops, a shop's products. The
list can live in a YAML or CSV file beside the post, so adding a photo is adding a line.

Photos rarely come in one shape. `ratio=4/3` crops every one to the same shape, so a tall lighthouse and a wide shore
sit in an even grid, two to a row here, and one under another on a phone:

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./first-packs/photos.yaml
      as: photos
---

### From the harbour

:: gallery[Four photos from the harbour, each cropped to 4:3] {items=(md.data.photos) ratio=4/3 columns=1 sm:columns=2}
```

`columns=1 sm:columns=2` is one card per row, then two from the `sm` width up. That `sm:` works on any param, and `md:`
and `lg:` too. Leave `columns` out, and the cards fill the row by their `min` width instead. `image-fit=contain` shows
each photo whole.

## A figure, beside its text, and columns

`figure` is an image with a caption under it. You write the caption once, and it is the image's alt text too, so the
two never drift apart. `columns` puts what you write side by side, and stacks it on a phone, which is also how a plain
reader shows it. Together, a photo sits beside the paragraph about it:

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-end}
:: figure[**Figure A**: Rows of apple trees on the slope above the town.] {src="first-packs/orchard.svg"}

:: #end
**The old orchard.** Forty trees on the slope above the town, some older than the church clock. Nobody owns the
apples: on the first Saturday of October the town picks them together, and what isn't eaten by Sunday goes to the
cider press at the bottom of the hill.
::: /columns
```

What comes first is the first column, and `:: #end` starts the last; `split=wide-end` makes it twice as wide. A
`:: #middle` makes three, for text that reads like a newspaper's, here the opening of _Alice's Adventures in
Wonderland_:

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns
Alice was beginning to get very tired of sitting by her sister on the bank, and of having nothing to do: once or
twice she had peeped into the book her sister was reading, but it had no pictures or conversations in it, "and what
is the use of a book," thought Alice "without pictures or conversations?"

:: #middle
So she was considering in her own mind (as well as she could, for the hot day made her feel very sleepy and stupid),
whether the pleasure of making a daisy-chain would be worth the trouble of getting up and picking the daisies, when
suddenly a White Rabbit with pink eyes ran close by her.

:: #end
There was nothing so very remarkable in that; nor did Alice think it so very much out of the way to hear the Rabbit
say to itself, "Oh dear! Oh dear! I shall be late!" But when the Rabbit actually took a watch out of its
waistcoat-pocket, and looked at it, and then hurried on, Alice started to her feet, and ran across the field after
it.
::: /columns
```

## Badges, and your own

`badge` is a small label in the text, as tall as the words around it. Its `tone` takes the page's colors, so it looks
right in a light theme and a dark one.

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The old search :badge[deprecated]{tone=subtle}, the map :badge[stable]{tone=strong}, the new search :badge[beta]{tone=accent}.
```

The part I like most: a project defines its own badges once, each a `badge` with its look fixed, and every document
then writes `:new[new]` with nothing to configure. The definitions are plain CompMD too, using the type params that
came with these packs (`font-size`, `font-weight`, `text-transform`, `letter-spacing`) and a gradient for the one
that should shout:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

```compmd component=new
---
:badge[{{md.fallback}}]{background="linear-gradient(90deg, #7c3aed, #db2777)" color="#ffffff" font-size=s font-weight=bold text-transform=uppercase letter-spacing=wide}
```

```compmd component=fixed
---
:badge[{{md.fallback}}]{background="#dafbe1" border="1px solid #1a7f37" color="#116329" font-size=s font-weight=bold}
```

```compmd component=breaking
---
:badge[{{md.fallback}}]{background="#ffebe9" border="1px solid #cf222e" color="#a40e26" font-size=s font-weight=bold text-transform=uppercase}
```

**Version 2.1**

- :new[new] Saved searches, from the search page.
- :fixed[fixed] The tide table showed yesterday's tides after midnight.
- :breaking[breaking] The old map links go to the new map.
````

In a plain Markdown reader those lines read "new Saved searches…", the badges' words in place. Put the definitions
in a file of their own and import it, and every page of a site has the same badges; the
[badge's README](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/badge/README.md) shows how.

## A sticky note

A `sticky-note` is a yellow note, pinned and slightly tilted, with a corner turned up. In the flow, it sits where you
write it. With an `inset-*`, it is pinned in its box, over the text, and with `sm:` in front, only on screens wide
enough for it: on a phone it goes back in the flow, after the text, where it doesn't cover anything. Its colors are
the classic ones, so it reads the same in a dark theme, and a use can change them: `{background=subtle color=accent}`.

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: box {padding=m sm:padding-inline-end=13rem border=subtle border-radius=m}
**The harbour walk.** Start at the lifeboat station and follow the old quay to the lighthouse. Back along the old
wall, the lamps come on an hour before the boats come in.

At the end of the wall, the steps down to the beach are slippery in the morning, so take the ramp by the café, and
come back up through the fish market, where the stalls close at four.

::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=1rem width=11rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
::: /box
```

## A timeline

`timeline` holds `event`s, each with a date, a title and whatever you want to say about it. The dates are text, so
`1897`, `May 3` and `Day 2` all work, and `tone=accent` marks the one that matters now.

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/timeline@0"
      names: [timeline, event]
---

::: timeline
::: event {date="March 2023" title="Taken down"}
The lamp posts go to the foundry for repair.
::: /event

::: event {date="October 2023" title="The first six"}
Back on the quay, by the lifeboat station.
::: /event

::: event {date="April 2024" title="The last of them" tone=accent}
Lit again on a spring evening, with a band.
::: /event
::: /timeline
```

Written as an `each` over a YAML file, the same timeline grows by one line per milestone.

## A rating, in a sentence

`rating` is the one of the nine with code: it draws its stars as SVG, in the host's sandbox, rounded to the nearest
half. Written inline, it sits in a sentence like a word:

```compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

The cheese was :rating[two of five]{value=2 width=5rem} at best, but the bread :rating[five of five]{value=5 width=5rem}.
```

In a plain Markdown reader, those are the words "two of five" and "five of five". Nothing is lost.

## What's under the hood

Open any of these packs and you'll find a short `index.comp.md`: the component's signature (its params and their
types), then its body, in the same Markdown you write posts in. Here is `card`, all of it:

````md
```compmd component=card {padding=m border=subtle border-radius=m}
slots: [default, image, footer]
---
{{md.slot.image}}

{{md.slot}}

{{md.slot.footer}}
```
````

That's the whole card. The rest of its box is the params any box takes, and a use can change them:
`::: card {background=subtle border=none}`.

Not every component can be Markdown, of course. A chart, a Mermaid diagram or a wireframe needs code, and those packs
run it in the sandbox, apart from the page and your data. Our [Mermaid pack](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/mermaid/README.md)
is one of them, and so is the `rating` above.

## Making your own

If one of these is almost what you need, copy it and change it. If you need something new, with code, one command
gets you a pack with a test page:

```sh
npm create @compmd/pack
```

Then `compmd-pack dev` serves your pack's demo, in the sandbox, as a host would show it, and `compmd-pack build`
checks it before you publish. The [Developer's Guide](https://compmd.dev/guides/developers/) walks through a first pack
step by step, and the [Author's Guide](https://compmd.dev/guides/authors/) covers boxes, grids and the params every
component takes.

## Conclusion

Most pages people write are made of a handful of patterns: cards, galleries, notes, timelines, columns. With these
nine packs, a CompMD document has them all, and the file still reads well in any Markdown reader. Every one of them is on
[GitHub](https://github.com/comp-markdown/compmd/tree/main/compmd-packs), with its README, its demo and its tests.

What should the next pack be? Tell me in the comments below, or in [an issue on GitHub](https://github.com/comp-markdown/compmd/issues).
If it's a pattern you keep rebuilding by hand, it's probably one somebody else keeps rebuilding too.

Until next time...

Hakan Serce
