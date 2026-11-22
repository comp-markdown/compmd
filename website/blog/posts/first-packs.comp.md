---
title: "The first packs: cards, galleries, notes and more"
date: 2026-11-23
summary: Nine components for the pages people actually write, a card, a gallery, a sticky note, a timeline and more, eight of them written in CompMD itself, with no code at all.
tags: [packs, components]
draft: true
---

The first thing I wanted to make with CompMD was a newsletter. Not a fancy one: a few stories, a row of photos, a note
about the next meeting, links to read next. Every one of those is a pattern you've seen a thousand times on the web,
and every one of them needs more than plain Markdown.

So before anything else, we made those patterns: nine components, each a **pack** on npm that a document imports by
name. This post is a tour. Each example below shows its source first, then what it draws, so you can see exactly what
you'd write.

The twist: eight of the nine have no code. They are written in CompMD itself, a few lines of Markdown with a signature
on top, so every CompMD processor renders them, with no sandbox and nothing to load. You could read one in a minute and
write your own in five.

## A card, and cards that are links

A `card` is a box with an image on top, then whatever you write, then a footer. The image is your own Markdown image,
so its path and its alt text are yours, and the title is your own heading, at the level your page needs.

````compmd {render=both}
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
````

For a "read next" row, `link-card` is a card that is a link: click anywhere on it and you follow it, while a screen
reader still names the link by its title. Put a few in a `grid` and they wrap, each as tall as the tallest.

````compmd {render=both}
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
````

## A gallery, from a list

A `gallery` is a grid of cards, one for each item of a list: a post's photos, a trip's stops, a shop's products. The
list can live in a YAML or CSV file beside the post, so adding a photo is adding a line.

Photos rarely come in one shape. `ratio=4/3` crops every one to the same shape, so a tall lighthouse and a wide shore
sit in an even grid:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/gallery@0"
      names: [gallery]
  data:
    - from: ./first-packs/photos.yaml
      as: photos
---

:: gallery[Four photos from the harbour, each cropped to 4:3] {items=(md.data.photos) min=10rem ratio=4/3}
````

`image-fit=contain` shows each photo whole instead, and `columns=1 md:columns=3` stacks the cards on a phone and puts
three side by side on a wider screen. That `md:` works on any param.

## A figure and its caption

`figure` is an image with a caption under it. You write the caption once, and it is the image's alt text too, so the
two never drift apart.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/figure@0"
      names: [figure]
---

:: figure[Rows of apple trees on the slope above the town.] {src="first-packs/orchard.svg" width=20rem}
````

## Badges, and your own

`badge` is a small label in the text, as tall as the words around it. Its `tone` takes the page's colours, so it looks
right in a light theme and a dark one.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/badge@0"
      names: [badge]
---

The old search :badge[deprecated]{tone=subtle}, the map :badge[stable]{tone=strong}, the new search :badge[beta]{tone=accent}.
````

The part I like most: a project defines its own badges once, a red `bug`, a green `feature`, in a file of their own,
and every document then writes `:bug[bug]` with nothing to configure. That file is plain CompMD too, using the type
params that came with these packs: `font-size=s`, `font-weight=bold`, `color=…`, `font-family=mono`. The
[badge's README](https://github.com/comp-markdown/compmd/blob/main/compmd-packs/badge/README.md) shows it.

## A sticky note

A `sticky-note` is a tinted, slightly tilted note with a shadow. In the flow, it sits where you write it. With an
`inset-*`, it is pinned in its box, over the text, and with `sm:` in front, only on screens wide enough for it: on a
phone it goes back in the flow, after the text, where it doesn't cover anything.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/sticky-note@0"
      names: [sticky-note]
---

::: box {padding=m border=subtle border-radius=m}
**The harbour walk.** Start at the lifeboat station and follow the old quay to the lighthouse. Back along the old
wall, the lamps come on an hour before the boats come in.

::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=1rem width=11rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
::: /box
````

## A timeline

`timeline` holds `event`s, each with a date, a title and whatever you want to say about it. The dates are text, so
`1897`, `May 3` and `Day 2` all work, and `tone=accent` marks the one that matters now.

````compmd {render=both}
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
````

Written as an `each` over a YAML file, the same timeline grows by a line a milestone.

## A rating, in a sentence

`rating` is the one of the nine with code: it draws its stars as SVG, in the host's sandbox, rounded to the nearest
half. Written inline, it sits in a sentence like a word:

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/rating@0"
      names: [rating]
---

The cheese was :rating[two of five]{value=2 width=5rem} at best, but the bread :rating[five of five]{value=5 width=5rem}.
````

In a plain Markdown reader, those are the words "two of five" and "five of five". Nothing is lost.

## Columns

`columns` puts what you write side by side, and stacks it on a phone, which is also how a plain reader shows it. What
comes first is the first column; `:: #end` starts the last, and `split=wide-end` makes it twice as wide.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-end}
**Ingredients**

- 4 sharp apples
- 100 g cold butter
- 100 g flour
- 100 g brown sugar

:: #end
**Method**

Slice the apples into a buttered dish. Rub the butter into the flour until it looks like breadcrumbs, stir in the
sugar, and scatter it over the apples without pressing down. Bake at 180 °C for 40 minutes, until golden.
::: /columns
````

Start and end follow the text's direction: in an Arabic post, the first column is on the right.

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
is one, and the `rating` above is another.

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
nine packs, a CompMD document has them, and the file still reads well in any Markdown reader. Every one of them is on
[GitHub](https://github.com/comp-markdown/compmd/tree/main/compmd-packs), with its README, its demo and its tests.

What should the next pack be? Tell me in the comments below, or in [an issue on GitHub](https://github.com/comp-markdown/compmd/issues).
If it's a pattern you keep rebuilding by hand, it's probably one somebody else keeps rebuilding too.

Until next time...

Hakan Serce
