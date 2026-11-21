---
md:
  export:
    pack: "@comp-markdown/sticky-note"
    version: "0.1.0"
    names: [sticky-note]
---

# @comp-markdown/sticky-note

`sticky-note` is a note pinned on the page: a tinted box, tilted a little, with a shadow, for a reminder beside a walk's directions, a tip in a recipe, a correction in a newsletter.

```md
::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=4rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
```

What you write in it is the note. Without a position it sits where it is written, in the flow. `inset-*` pins it in its nearest box (spec §9.2): `inset-inline-end=1rem inset-block-start=4rem` puts it 1rem from the box's end and 4rem from its top, over what is there; the box is as tall as what stays in its flow. Written from `sm` up, `sm:inset-…`, it is pinned where its box is wide and in the flow on a phone, where there is no room beside the text.

It is 14rem wide, never wider than its column; a use changes its box as any box's, `{rotate=3deg background=subtle width=10rem}`. It prints as it shows. In a plain Markdown reader, a note is its text.

```compmd component=sticky-note {padding=m background=accent shadow=m rotate=-2deg width=14rem max-width=100%}
slots: [default]
---
{{md.slot}}
```
